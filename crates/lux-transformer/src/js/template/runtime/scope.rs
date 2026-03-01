use oxc_ast::{
    AstBuilder, NONE,
    ast::{BindingPattern, Expression, FormalParameterKind, FunctionType, IdentifierReference},
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::SPAN;
use rustc_hash::FxHashSet;

#[derive(Default, Clone)]
pub(crate) struct RuntimeScope {
    local_bindings: FxHashSet<String>,
}

impl RuntimeScope {
    pub(super) fn contains(&self, name: &str) -> bool {
        self.local_bindings.contains(name)
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
}

pub(super) fn resolve_expression<'a>(
    ast: AstBuilder<'a>,
    expression: Expression<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
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
