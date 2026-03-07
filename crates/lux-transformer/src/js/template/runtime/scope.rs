use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{
        AccessorProperty, ArrowFunctionExpression, BindingPattern, CallExpression, CatchParameter,
        Class, Expression, FormalParameter, FormalParameterKind, Function, FunctionType,
        IdentifierReference, MethodDefinition, PropertyDefinition, VariableDeclarator,
    },
};
use oxc_ast_visit::{Visit, VisitMut, walk, walk_mut};
use oxc_span::SPAN;
use oxc_syntax::scope::ScopeFlags;
use rustc_hash::FxHashSet;

#[derive(Default, Clone)]
pub(crate) struct RuntimeScope {
    local_bindings: FxHashSet<String>,
    css_scope: Option<String>,
    store_subscriptions: bool,
}

impl RuntimeScope {
    pub(super) fn contains(&self, name: &str) -> bool {
        self.local_bindings.contains(name)
    }

    pub(super) fn css_scope(&self) -> Option<&str> {
        self.css_scope.as_deref()
    }

    pub(crate) fn from_names<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut scope = Self::default();
        for name in names {
            scope.local_bindings.insert(name.as_ref().to_owned());
        }
        scope
    }

    pub(super) fn with_binding_pattern(&self, pattern: &BindingPattern<'_>) -> Self {
        let mut next = self.clone();
        collect_binding_pattern_names(pattern, &mut next.local_bindings);
        next
    }

    pub(super) fn with_name(&self, name: &str) -> Self {
        let mut next = self.clone();
        next.local_bindings.insert(name.to_string());
        next
    }

    pub(crate) fn with_css_scope(&self, css_scope: Option<&str>) -> Self {
        let mut next = self.clone();
        next.css_scope = css_scope.map(ToOwned::to_owned);
        next
    }

    pub(crate) fn with_store_subscriptions(&self, enabled: bool) -> Self {
        let mut next = self.clone();
        next.store_subscriptions = enabled;
        next
    }
}

pub(super) fn resolve_expression<'a>(
    ast: AstBuilder<'a>,
    expression: Expression<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let expression = strip_typescript_expression(ast, expression);
    let expression = if scope.store_subscriptions {
        rewrite_store_subscriptions(ast, expression, scope)
    } else {
        expression
    };
    let mut collector = IdentifierCollector::default();
    collector.visit_expression(&expression);

    let mut names = collector
        .names
        .into_iter()
        .filter(|name| !scope.contains(name))
        .filter(|name| !is_runtime_global(name))
        .collect::<Vec<_>>();

    if names.is_empty() {
        return expression;
    }

    names.sort_unstable();
    names.dedup();

    let mut properties = ast.vec_with_capacity(names.len());
    for name in names {
        properties.push(ast.binding_property(
            SPAN,
            ast.property_key_static_identifier(SPAN, ast.ident(name)),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident(name)),
            true,
            false,
        ));
    }

    let props_pattern = ast.binding_pattern_object_pattern(SPAN, properties, NONE);
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(ast.formal_parameter(
            SPAN,
            ast.vec(),
            props_pattern,
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        )),
        NONE,
    );
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(expression))),
    );
    let resolver = ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        None,
        false,
        false,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(body),
    );

    ast.expression_call(
        SPAN,
        resolver,
        NONE,
        ast.vec1(ast.expression_identifier(SPAN, ast.ident("_props")).into()),
        false,
    )
}

fn rewrite_store_subscriptions<'a>(
    ast: AstBuilder<'a>,
    mut expression: Expression<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut rewriter = StoreSubscriptionRewriter { ast, scope };
    rewriter.visit_expression(&mut expression);
    expression
}

#[derive(Default)]
struct IdentifierCollector<'a> {
    names: FxHashSet<&'a str>,
}

impl<'a> Visit<'a> for IdentifierCollector<'a> {
    fn visit_expression(&mut self, it: &Expression<'a>) {
        walk::walk_expression(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.names.insert(it.name.as_str());
    }
}

pub(super) fn collect_binding_pattern_names(
    pattern: &BindingPattern<'_>,
    names: &mut FxHashSet<String>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.insert(identifier.name.as_str().to_string());
        }
        BindingPattern::ObjectPattern(pattern) => {
            for property in &pattern.properties {
                collect_binding_pattern_names(&property.value, names);
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(pattern) => {
            for element in &pattern.elements {
                if let Some(element) = element {
                    collect_binding_pattern_names(element, names);
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(pattern) => {
            collect_binding_pattern_names(&pattern.left, names);
        }
    }
}

fn is_runtime_global(name: &str) -> bool {
    matches!(
        name,
        "_props"
            | "__lux_store_get"
            | "__lux_store_values"
            | "undefined"
            | "Infinity"
            | "NaN"
            | "Math"
            | "Number"
            | "String"
            | "Boolean"
            | "Object"
            | "Array"
            | "Date"
            | "JSON"
            | "RegExp"
            | "Map"
            | "Set"
            | "WeakMap"
            | "WeakSet"
            | "Promise"
            | "Symbol"
            | "BigInt"
            | "console"
            | "window"
            | "document"
            | "globalThis"
    )
}

pub(super) fn is_valid_js_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn strip_typescript_expression<'a>(
    ast: AstBuilder<'a>,
    mut expression: Expression<'a>,
) -> Expression<'a> {
    let mut eraser = TypeScriptExpressionStripper { ast };
    eraser.visit_expression(&mut expression);
    expression
}

fn strip_typescript_expression_wrapper<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
) -> Option<Expression<'a>> {
    match expression {
        Expression::TSAsExpression(wrapper) => Some(wrapper.expression.clone_in(ast.allocator)),
        Expression::TSSatisfiesExpression(wrapper) => {
            Some(wrapper.expression.clone_in(ast.allocator))
        }
        Expression::TSTypeAssertion(wrapper) => Some(wrapper.expression.clone_in(ast.allocator)),
        Expression::TSNonNullExpression(wrapper) => {
            Some(wrapper.expression.clone_in(ast.allocator))
        }
        Expression::TSInstantiationExpression(wrapper) => {
            Some(wrapper.expression.clone_in(ast.allocator))
        }
        _ => None,
    }
}

struct TypeScriptExpressionStripper<'a> {
    ast: AstBuilder<'a>,
}

struct StoreSubscriptionRewriter<'ast, 'scope> {
    ast: AstBuilder<'ast>,
    scope: &'scope RuntimeScope,
}

impl<'ast> VisitMut<'ast> for StoreSubscriptionRewriter<'ast, '_> {
    fn visit_expression(&mut self, expression: &mut Expression<'ast>) {
        walk_mut::walk_expression(self, expression);

        let Expression::Identifier(identifier) = expression else {
            return;
        };
        let name = identifier.name.as_str();
        if !name.starts_with('$') || name.starts_with("$$") || name.len() < 2 {
            return;
        }

        let store_name = &name[1..];
        if !self.scope.contains(store_name) {
            return;
        }

        *expression = self.ast.expression_call(
            SPAN,
            self.ast
                .expression_identifier(SPAN, self.ast.ident("__lux_store_get")),
            NONE,
            self.ast.vec_from_array([
                self.ast
                    .expression_identifier(SPAN, self.ast.ident("__lux_store_values"))
                    .into(),
                self.ast
                    .expression_string_literal(SPAN, self.ast.atom(name), None)
                    .into(),
                self.ast
                    .expression_identifier(SPAN, self.ast.ident(store_name))
                    .into(),
            ]),
            false,
        );
    }
}

impl<'a> VisitMut<'a> for TypeScriptExpressionStripper<'a> {
    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        while let Some(inner) = strip_typescript_expression_wrapper(self.ast, expression) {
            *expression = inner;
        }
        walk_mut::walk_expression(self, expression);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        declarator.type_annotation = None;
        declarator.definite = false;
        walk_mut::walk_variable_declarator(self, declarator);
    }

    fn visit_function(&mut self, function: &mut Function<'a>, flags: ScopeFlags) {
        function.type_parameters = None;
        function.this_param = None;
        function.return_type = None;
        walk_mut::walk_function(self, function, flags);
    }

    fn visit_formal_parameter(&mut self, parameter: &mut FormalParameter<'a>) {
        parameter.decorators = self.ast.vec();
        parameter.type_annotation = None;
        parameter.optional = false;
        parameter.accessibility = None;
        walk_mut::walk_formal_parameter(self, parameter);
    }

    fn visit_catch_parameter(&mut self, parameter: &mut CatchParameter<'a>) {
        parameter.type_annotation = None;
        walk_mut::walk_catch_parameter(self, parameter);
    }

    fn visit_arrow_function_expression(&mut self, expression: &mut ArrowFunctionExpression<'a>) {
        expression.type_parameters = None;
        expression.return_type = None;
        walk_mut::walk_arrow_function_expression(self, expression);
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        class.type_parameters = None;
        class.super_type_arguments = None;
        class.implements = self.ast.vec();
        walk_mut::walk_class(self, class);
    }

    fn visit_method_definition(&mut self, definition: &mut MethodDefinition<'a>) {
        definition.r#override = false;
        definition.optional = false;
        definition.accessibility = None;
        walk_mut::walk_method_definition(self, definition);
    }

    fn visit_property_definition(&mut self, definition: &mut PropertyDefinition<'a>) {
        definition.type_annotation = None;
        definition.declare = false;
        definition.r#override = false;
        definition.optional = false;
        definition.definite = false;
        definition.readonly = false;
        definition.accessibility = None;
        walk_mut::walk_property_definition(self, definition);
    }

    fn visit_accessor_property(&mut self, property: &mut AccessorProperty<'a>) {
        property.type_annotation = None;
        property.r#override = false;
        property.definite = false;
        property.accessibility = None;
        walk_mut::walk_accessor_property(self, property);
    }

    fn visit_call_expression(&mut self, expression: &mut CallExpression<'a>) {
        expression.type_arguments = None;
        walk_mut::walk_call_expression(self, expression);
    }

    fn visit_new_expression(&mut self, expression: &mut oxc_ast::ast::NewExpression<'a>) {
        expression.type_arguments = None;
        walk_mut::walk_new_expression(self, expression);
    }
}
