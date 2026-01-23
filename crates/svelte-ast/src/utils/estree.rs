use serde::ser::Error as SerError;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use swc_ecma_ast as swc;

/// Wrapper for serializing an expression in ESTree format via `serialize_map`.
pub struct ExprWrapper<'a>(pub &'a Box<swc::Expr>);

impl<'a> Serialize for ExprWrapper<'a> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(self.0.as_ref()).map_err(S::Error::custom)?;
        let transformed = transform_value(value);
        transformed.serialize(s)
    }
}

/// Serialize a `Box<swc::Expr>` in ESTree format.
pub fn serialize_boxed_expr<S: Serializer>(expr: &Box<swc::Expr>, s: S) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(expr.as_ref()).map_err(S::Error::custom)?;
    let transformed = transform_value(value);
    transformed.serialize(s)
}

/// Serialize an `Option<Box<swc::Expr>>` in ESTree format.
pub fn serialize_opt_expr<S: Serializer>(
    expr: &Option<Box<swc::Expr>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match expr {
        Some(e) => {
            let value = serde_json::to_value(e.as_ref()).map_err(S::Error::custom)?;
            let transformed = transform_value(value);
            transformed.serialize(s)
        }
        None => s.serialize_none(),
    }
}

/// Serialize a `Box<swc::Pat>` in ESTree format.
pub fn serialize_boxed_pat<S: Serializer>(pat: &Box<swc::Pat>, s: S) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(pat.as_ref()).map_err(S::Error::custom)?;
    let transformed = transform_value(value);
    transformed.serialize(s)
}

/// Serialize an `Option<Box<swc::Pat>>` in ESTree format.
pub fn serialize_opt_pat<S: Serializer>(
    pat: &Option<Box<swc::Pat>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match pat {
        Some(p) => serialize_boxed_pat(p, s),
        None => s.serialize_none(),
    }
}

/// Serialize a `Vec<swc::Pat>` in ESTree format.
pub fn serialize_pats<S: Serializer>(pats: &[swc::Pat], s: S) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(pats).map_err(S::Error::custom)?;
    let transformed = transform_value(value);
    transformed.serialize(s)
}

/// Serialize a `Box<swc::VarDecl>` in ESTree format.
pub fn serialize_boxed_var_decl<S: Serializer>(
    decl: &Box<swc::VarDecl>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(decl.as_ref()).map_err(S::Error::custom)?;
    let transformed = transform_value(value);
    transformed.serialize(s)
}

/// Serialize a `Box<swc::Ident>` in ESTree format.
pub fn serialize_boxed_ident<S: Serializer>(
    ident: &Box<swc::Ident>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(ident.as_ref()).map_err(S::Error::custom)?;
    let transformed = transform_value(value);
    transformed.serialize(s)
}

/// Serialize a `Vec<swc::Ident>` in ESTree format.
pub fn serialize_idents<S: Serializer>(idents: &[swc::Ident], s: S) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(idents).map_err(S::Error::custom)?;
    let transformed = transform_value(value);
    transformed.serialize(s)
}

/// Serialize a `swc::Program` in ESTree format.
pub fn serialize_program<S: Serializer>(program: &swc::Program, s: S) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(program).map_err(S::Error::custom)?;
    let transformed = transform_value(value);
    transformed.serialize(s)
}

/// Wrapper for serializing a Program reference in ESTree format.
/// Used in custom Serialize impls where `serialize_with` isn't applicable.
pub struct ProgramRef<'a>(pub &'a swc::Program);

impl<'a> Serialize for ProgramRef<'a> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(self.0).map_err(S::Error::custom)?;
        let transformed = transform_value(value);
        transformed.serialize(s)
    }
}

/// Wrapper for serializing a Program with leadingComments/trailingComments attached.
/// Comments are positioned relative to body statements:
/// - Before first statement → leadingComments on Program
/// - After last statement → trailingComments on Program
pub struct ProgramWithComments<'a> {
    pub program: &'a swc::Program,
    pub comments: &'a [crate::text::JsComment],
    pub content_start: usize,
    pub content_end: usize,
}

impl<'a> Serialize for ProgramWithComments<'a> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(self.program).map_err(S::Error::custom)?;
        let mut transformed = transform_value(value);

        // Program.start/end should be the content boundaries (between > and </script>)
        if let Value::Object(ref mut obj) = transformed {
            if self.content_start > 0 || self.content_end > 0 {
                obj.insert(
                    "start".to_string(),
                    Value::Number(self.content_start.into()),
                );
                obj.insert("end".to_string(), Value::Number(self.content_end.into()));
            }
        }

        if !self.comments.is_empty() {
            if let Value::Object(ref mut obj) = transformed {
                let body_is_empty = obj
                    .get("body")
                    .and_then(|b| b.as_array())
                    .map(|arr| arr.is_empty())
                    .unwrap_or(true);

                if body_is_empty {
                    // No statements: all comments are trailingComments
                    let comments_val: Vec<Value> = self
                        .comments
                        .iter()
                        .map(|c| serde_json::to_value(c).map_err(S::Error::custom))
                        .collect::<Result<_, _>>()?;
                    obj.insert("trailingComments".to_string(), Value::Array(comments_val));
                } else {
                    // Use first statement start as boundary
                    let body_start = obj
                        .get("body")
                        .and_then(|b| b.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|node| node.get("start"))
                        .and_then(|s| s.as_u64())
                        .unwrap_or(u64::MAX);

                    let mut leading = Vec::new();
                    let mut trailing = Vec::new();

                    for comment in self.comments {
                        let c_val = serde_json::to_value(comment).map_err(S::Error::custom)?;
                        if (comment.span.start as u64) < body_start {
                            leading.push(c_val);
                        } else {
                            trailing.push(c_val);
                        }
                    }

                    if !leading.is_empty() {
                        obj.insert("leadingComments".to_string(), Value::Array(leading));
                    }
                    if !trailing.is_empty() {
                        obj.insert("trailingComments".to_string(), Value::Array(trailing));
                    }
                }
            }
        }

        transformed.serialize(s)
    }
}

/// Transform a serde_json::Value from SWC format to ESTree format.
fn transform_value(value: Value) -> Value {
    match value {
        Value::Object(obj) => transform_node(obj),
        Value::Array(arr) => Value::Array(arr.into_iter().map(transform_value).collect()),
        other => other,
    }
}

/// Transform a JSON object node from SWC format to ESTree format.
fn transform_node(mut obj: Map<String, Value>) -> Value {
    // 1. Flatten span: { start, end, ctxt } -> top-level start, end
    if let Some(Value::Object(span)) = obj.remove("span") {
        if let Some(start) = span.get("start") {
            obj.entry("start".to_string()).or_insert(start.clone());
        }
        if let Some(end) = span.get("end") {
            obj.entry("end".to_string()).or_insert(end.clone());
        }
    }

    // 2. Remove ctxt from any level
    obj.remove("ctxt");

    // 3. Type-specific transformations
    let node_type = obj.get("type").and_then(|v| v.as_str()).map(String::from);
    if let Some(ref t) = node_type {
        match t.as_str() {
            // --- Identifiers ---
            "Identifier" => {
                if let Some(v) = obj.remove("value") {
                    obj.insert("name".to_string(), v);
                }
                obj.remove("optional");
                // Remove null typeAnnotation
                if obj.get("typeAnnotation") == Some(&Value::Null) {
                    obj.remove("typeAnnotation");
                }
            }

            // --- Literals: SWC *Literal → ESTree Literal ---
            "NumericLiteral" => {
                obj.insert("type".to_string(), Value::String("Literal".to_string()));
                // Convert float to int when possible (5.0 → 5)
                if let Some(Value::Number(n)) = obj.get("value") {
                    if let Some(f) = n.as_f64() {
                        if f.fract() == 0.0 && f.abs() < (i64::MAX as f64) {
                            obj.insert("value".to_string(), Value::Number((f as i64).into()));
                        }
                    }
                }
            }
            "StringLiteral" => {
                obj.insert("type".to_string(), Value::String("Literal".to_string()));
            }
            "BooleanLiteral" | "BoolLit" => {
                obj.insert("type".to_string(), Value::String("Literal".to_string()));
                // Add raw field from value
                if let Some(Value::Bool(b)) = obj.get("value") {
                    let raw = if *b { "true" } else { "false" };
                    obj.entry("raw".to_string())
                        .or_insert(Value::String(raw.to_string()));
                }
            }
            "NullLiteral" => {
                obj.insert("type".to_string(), Value::String("Literal".to_string()));
                obj.insert("value".to_string(), Value::Null);
                obj.insert("raw".to_string(), Value::String("null".to_string()));
            }
            "BigIntLiteral" => {
                obj.insert("type".to_string(), Value::String("Literal".to_string()));
            }
            "RegExpLiteral" => {
                obj.insert("type".to_string(), Value::String("Literal".to_string()));
                let pattern = obj
                    .remove("pattern")
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                let flags = obj
                    .remove("flags")
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                obj.insert("value".to_string(), Value::Object(Map::new()));
                let mut regex = Map::new();
                regex.insert("pattern".to_string(), Value::String(pattern.clone()));
                regex.insert("flags".to_string(), Value::String(flags.clone()));
                obj.insert("regex".to_string(), Value::Object(regex));
                obj.insert(
                    "raw".to_string(),
                    Value::String(format!("/{}/{}", pattern, flags)),
                );
            }

            // --- Expressions ---
            "CallExpression" | "OptionalCallExpression" => {
                let is_optional = t == "OptionalCallExpression";
                // SWC may serialize as "args" or "arguments"
                let args = obj.remove("args").or_else(|| obj.remove("arguments"));
                if let Some(args) = args {
                    let unwrapped = unwrap_expr_or_spread(args);
                    obj.insert("arguments".to_string(), unwrapped);
                }
                // ESTree requires optional field
                obj.entry("optional".to_string())
                    .or_insert(Value::Bool(is_optional));
                // Remove null optional fields
                remove_if_null(&mut obj, "typeArguments");
                remove_if_null(&mut obj, "typeParameters");
            }
            "NewExpression" => {
                let args = obj.remove("args").or_else(|| obj.remove("arguments"));
                if let Some(args) = args {
                    let unwrapped = unwrap_expr_or_spread(args);
                    obj.insert("arguments".to_string(), unwrapped);
                }
            }
            "BinaryExpression" | "LogicalExpression" | "AssignmentExpression" => {
                if let Some(op) = obj.remove("op") {
                    obj.insert("operator".to_string(), op);
                }
            }
            "UnaryExpression" | "UpdateExpression" => {
                if let Some(op) = obj.remove("op") {
                    obj.insert("operator".to_string(), op);
                }
                if let Some(arg) = obj.remove("arg") {
                    obj.insert("argument".to_string(), arg);
                }
            }
            "ConditionalExpression" | "IfStatement" => {
                if let Some(cons) = obj.remove("cons") {
                    obj.insert("consequent".to_string(), cons);
                }
                if let Some(alt) = obj.remove("alt") {
                    obj.insert("alternate".to_string(), alt);
                }
            }
            "TemplateLiteral" => {
                if let Some(exprs) = obj.remove("exprs") {
                    obj.insert("expressions".to_string(), exprs);
                }
            }
            "TemplateElement" => {
                // SWC has cooked/raw at top level, ESTree wraps in value: { cooked, raw }
                let cooked = obj.remove("cooked");
                let raw = obj.remove("raw");
                let mut value_obj = Map::new();
                if let Some(c) = cooked {
                    value_obj.insert("cooked".to_string(), c);
                }
                if let Some(r) = raw {
                    value_obj.insert("raw".to_string(), r);
                }
                obj.insert("value".to_string(), Value::Object(value_obj));
            }
            "TaggedTemplateExpression" => {
                if let Some(exprs) = obj.remove("exprs") {
                    obj.insert("expressions".to_string(), exprs);
                }
            }
            "ParenthesisExpression" | "ParenExpr" => {
                // Not a valid ESTree type - unwrap to inner expression
                if let Some(inner) = obj.remove("expression").or_else(|| obj.remove("expr")) {
                    return transform_value(inner);
                }
            }
            "ObjectExpression" => {
                if let Some(props) = obj.remove("props") {
                    obj.insert("properties".to_string(), props);
                }
            }
            "ArrayExpression" => {
                if let Some(elems) = obj.remove("elems") {
                    obj.insert("elements".to_string(), elems);
                }
            }
            "MemberExpression" | "OptionalMemberExpression" => {
                if let Some(o) = obj.remove("obj") {
                    obj.insert("object".to_string(), o);
                }
                if let Some(p) = obj.remove("prop") {
                    obj.insert("property".to_string(), p);
                }
            }
            "SpreadElement" => {
                if let Some(expr) = obj.remove("expr") {
                    obj.insert("argument".to_string(), expr);
                }
            }
            "YieldExpression" => {
                if let Some(arg) = obj.remove("arg") {
                    obj.insert("argument".to_string(), arg);
                }
            }
            "AwaitExpression" => {
                if let Some(arg) = obj.remove("arg") {
                    obj.insert("argument".to_string(), arg);
                }
            }
            "SequenceExpression" => {
                if let Some(exprs) = obj.remove("exprs") {
                    obj.insert("expressions".to_string(), exprs);
                }
            }

            // --- Statements ---
            "BlockStatement" => {
                if let Some(stmts) = obj.remove("stmts") {
                    obj.insert("body".to_string(), stmts);
                }
            }
            "ExpressionStatement" => {
                if let Some(expr) = obj.remove("expr") {
                    obj.insert("expression".to_string(), expr);
                }
            }
            "ReturnStatement" => {
                if let Some(arg) = obj.remove("arg") {
                    obj.insert("argument".to_string(), arg);
                }
            }
            "ThrowStatement" => {
                if let Some(arg) = obj.remove("arg") {
                    obj.insert("argument".to_string(), arg);
                }
            }
            "WhileStatement" | "DoWhileStatement" => {
                if let Some(cons) = obj.remove("cons") {
                    obj.insert("consequent".to_string(), cons);
                }
            }
            "ForStatement" => {
                if let Some(update) = obj.remove("update") {
                    if update != Value::Null {
                        obj.insert("update".to_string(), update);
                    }
                }
            }
            "ForInStatement" | "ForOfStatement" => {
                if let Some(left) = obj.remove("left") {
                    obj.insert("left".to_string(), left);
                }
            }
            "SwitchStatement" => {
                if let Some(cases) = obj.remove("cases") {
                    obj.insert("cases".to_string(), cases);
                }
            }
            "SwitchCase" => {
                if let Some(cons) = obj.remove("cons") {
                    obj.insert("consequent".to_string(), cons);
                }
            }
            "TryStatement" => {
                if let Some(handler) = obj.remove("handler") {
                    obj.insert("handler".to_string(), handler);
                }
            }
            "LabeledStatement" => {
                if let Some(body) = obj.remove("body") {
                    obj.insert("body".to_string(), body);
                }
            }

            // --- Declarations ---
            "VariableDeclaration" => {
                if let Some(decls) = obj.remove("decls") {
                    obj.insert("declarations".to_string(), decls);
                }
                obj.remove("declare");
            }
            "VariableDeclarator" => {
                if let Some(name) = obj.remove("name") {
                    obj.insert("id".to_string(), name);
                }
            }
            "FunctionDeclaration" | "FunctionExpression" => {
                if let Some(ident) = obj.remove("ident") {
                    obj.insert("id".to_string(), ident);
                }
                // Remove null id
                if obj.get("id") == Some(&Value::Null) {
                    // Keep null for FunctionExpression (valid), but some need it
                }
                obj.remove("declare");
            }
            "ArrowFunctionExpression" => {
                // SWC uses "params" with Pat wrappers, ESTree uses flat params
            }
            "ClassDeclaration" | "ClassExpression" => {
                if let Some(ident) = obj.remove("ident") {
                    obj.insert("id".to_string(), ident);
                }
                if let Some(sc) = obj.remove("superClass") {
                    obj.insert("superClass".to_string(), sc);
                }
                obj.remove("declare");
                obj.remove("isAbstract");
            }

            // --- Patterns ---
            "ObjectPattern" => {
                if let Some(props) = obj.remove("props") {
                    obj.insert("properties".to_string(), props);
                }
                obj.remove("optional");
                if obj.get("typeAnnotation") == Some(&Value::Null) {
                    obj.remove("typeAnnotation");
                }
            }
            "ArrayPattern" => {
                if let Some(elems) = obj.remove("elems") {
                    obj.insert("elements".to_string(), elems);
                }
                obj.remove("optional");
                if obj.get("typeAnnotation") == Some(&Value::Null) {
                    obj.remove("typeAnnotation");
                }
            }
            "AssignmentPattern" => {
                if obj.get("typeAnnotation") == Some(&Value::Null) {
                    obj.remove("typeAnnotation");
                }
            }
            "RestElement" => {
                if let Some(arg) = obj.remove("arg") {
                    obj.insert("argument".to_string(), arg);
                }
                if obj.get("typeAnnotation") == Some(&Value::Null) {
                    obj.remove("typeAnnotation");
                }
            }

            // --- Properties ---
            "KeyValueProperty" => {
                obj.insert("type".to_string(), Value::String("Property".to_string()));
                obj.insert("kind".to_string(), Value::String("init".to_string()));
                obj.insert("method".to_string(), Value::Bool(false));
                obj.insert("shorthand".to_string(), Value::Bool(false));
                obj.insert("computed".to_string(), Value::Bool(false));
            }
            "KeyValuePatternProperty" => {
                obj.insert("type".to_string(), Value::String("Property".to_string()));
                obj.insert("kind".to_string(), Value::String("init".to_string()));
                obj.insert("method".to_string(), Value::Bool(false));
                obj.insert("shorthand".to_string(), Value::Bool(false));
                obj.insert("computed".to_string(), Value::Bool(false));
            }
            "AssignmentPatternProperty" => {
                // SWC's AssignPatProp → ESTree Property (shorthand: true)
                // key is the identifier, value is the optional default
                obj.insert("type".to_string(), Value::String("Property".to_string()));
                obj.insert("kind".to_string(), Value::String("init".to_string()));
                obj.insert("method".to_string(), Value::Bool(false));
                obj.insert("shorthand".to_string(), Value::Bool(true));
                obj.insert("computed".to_string(), Value::Bool(false));

                let key = obj.get("key").cloned();
                let default_value = obj.remove("value");

                // Helper: extract start/end from raw SWC node (may be in span.start or top-level start)
                let get_start = |val: &Value| -> Option<Value> {
                    val.as_object().and_then(|o| {
                        o.get("start").cloned().or_else(|| {
                            o.get("span")
                                .and_then(|s| s.as_object())
                                .and_then(|s| s.get("start"))
                                .cloned()
                        })
                    })
                };
                let get_end = |val: &Value| -> Option<Value> {
                    val.as_object().and_then(|o| {
                        o.get("end").cloned().or_else(|| {
                            o.get("span")
                                .and_then(|s| s.as_object())
                                .and_then(|s| s.get("end"))
                                .cloned()
                        })
                    })
                };

                if let Some(ref key_val) = key {
                    match &default_value {
                        Some(Value::Null) | None => {
                            // No default: value = key (same identifier)
                            obj.insert("value".to_string(), key_val.clone());
                        }
                        Some(default_expr) => {
                            // Has default: value = AssignmentPattern { left: key, right: default }
                            let mut assign_pat = Map::new();
                            assign_pat.insert(
                                "type".to_string(),
                                Value::String("AssignmentPattern".to_string()),
                            );
                            assign_pat.insert("left".to_string(), key_val.clone());
                            assign_pat.insert("right".to_string(), default_expr.clone());
                            // Span: start from key, end from default
                            if let Some(start) = get_start(key_val) {
                                assign_pat.insert("start".to_string(), start);
                            }
                            if let Some(end) = get_end(default_expr) {
                                assign_pat.insert("end".to_string(), end.clone());
                                // Update Property end to include default value
                                obj.insert("end".to_string(), end);
                            }
                            obj.insert("value".to_string(), Value::Object(assign_pat));
                        }
                    }
                }
            }
            "AssignmentProperty" => {
                obj.insert("type".to_string(), Value::String("Property".to_string()));
                obj.insert("kind".to_string(), Value::String("init".to_string()));
                obj.insert("method".to_string(), Value::Bool(false));
            }
            "GetterProperty" => {
                obj.insert("type".to_string(), Value::String("Property".to_string()));
                obj.insert("kind".to_string(), Value::String("get".to_string()));
            }
            "SetterProperty" => {
                obj.insert("type".to_string(), Value::String("Property".to_string()));
                obj.insert("kind".to_string(), Value::String("set".to_string()));
            }
            "MethodProperty" => {
                obj.insert("type".to_string(), Value::String("Property".to_string()));
                obj.insert("kind".to_string(), Value::String("init".to_string()));
                obj.insert("method".to_string(), Value::Bool(true));
            }

            // --- Imports/Exports ---
            "ImportDeclaration" => {
                if let Some(specifiers) = obj.remove("specifiers") {
                    obj.insert("specifiers".to_string(), specifiers);
                }
            }
            "ExportDefaultDeclaration" => {
                if let Some(decl) = obj.remove("decl") {
                    obj.insert("declaration".to_string(), decl);
                }
            }
            "ExportNamedDeclaration" | "ExportDeclaration" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("ExportNamedDeclaration".to_string()),
                );
                if let Some(decl) = obj.remove("decl") {
                    obj.insert("declaration".to_string(), decl);
                }
            }

            // --- TypeScript: Ts* → TS* ---
            "TsTypeAnnotation" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeAnnotation".to_string()),
                );
                if let Some(ta) = obj.remove("typeAnnotation") {
                    obj.insert("typeAnnotation".to_string(), ta);
                }
            }
            "TsTypeReference" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeReference".to_string()),
                );
            }
            "TsKeywordType" => {
                // {type: "TsKeywordType", kind: "number"} → {type: "TSNumberKeyword"}
                let kind = obj
                    .remove("kind")
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                let ts_type = match kind.as_str() {
                    "number" => "TSNumberKeyword",
                    "string" => "TSStringKeyword",
                    "boolean" => "TSBooleanKeyword",
                    "void" => "TSVoidKeyword",
                    "undefined" => "TSUndefinedKeyword",
                    "null" => "TSNullKeyword",
                    "any" => "TSAnyKeyword",
                    "never" => "TSNeverKeyword",
                    "unknown" => "TSUnknownKeyword",
                    "object" => "TSObjectKeyword",
                    "bigint" => "TSBigIntKeyword",
                    "symbol" => "TSSymbolKeyword",
                    "intrinsic" => "TSIntrinsicKeyword",
                    _ => "TSAnyKeyword",
                };
                obj.insert("type".to_string(), Value::String(ts_type.to_string()));
            }
            "TsArrayType" => {
                obj.insert("type".to_string(), Value::String("TSArrayType".to_string()));
                if let Some(elem) = obj.remove("elemType") {
                    obj.insert("elementType".to_string(), elem);
                }
            }
            "TsUnionType" | "TsUnionOrIntersectionType" => {
                obj.insert("type".to_string(), Value::String("TSUnionType".to_string()));
            }
            "TsIntersectionType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSIntersectionType".to_string()),
                );
            }
            "TsFunctionType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSFunctionType".to_string()),
                );
            }
            "TsTypeLiteral" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeLiteral".to_string()),
                );
                if let Some(members) = obj.remove("members") {
                    obj.insert("members".to_string(), members);
                }
            }
            "TsPropertySignature" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSPropertySignature".to_string()),
                );
            }
            "TsMethodSignature" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSMethodSignature".to_string()),
                );
            }
            "TsTypeAliasDeclaration" | "TsTypeAliasDecl" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeAliasDeclaration".to_string()),
                );
                obj.remove("declare");
            }
            "TsInterfaceDeclaration" | "TsInterfaceDecl" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSInterfaceDeclaration".to_string()),
                );
                obj.remove("declare");
            }
            "TsAsExpression" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSAsExpression".to_string()),
                );
            }
            "TsTypeAssertion" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeAssertion".to_string()),
                );
            }
            "TsNonNullExpression" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSNonNullExpression".to_string()),
                );
            }
            "TsParameterProperty" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSParameterProperty".to_string()),
                );
            }
            "TsQualifiedName" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSQualifiedName".to_string()),
                );
            }
            "TsTupleType" => {
                obj.insert("type".to_string(), Value::String("TSTupleType".to_string()));
            }
            "TsLiteralType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSLiteralType".to_string()),
                );
            }
            "TsTypeQuery" => {
                obj.insert("type".to_string(), Value::String("TSTypeQuery".to_string()));
            }
            "TsTypePredicate" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypePredicate".to_string()),
                );
            }
            "TsConditionalType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSConditionalType".to_string()),
                );
            }
            "TsIndexedAccessType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSIndexedAccessType".to_string()),
                );
            }
            "TsMappedType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSMappedType".to_string()),
                );
            }
            "TsEnumDeclaration" | "TsEnumDecl" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSEnumDeclaration".to_string()),
                );
                obj.remove("declare");
            }
            "TsModuleDeclaration" | "TsModuleDecl" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSModuleDeclaration".to_string()),
                );
                obj.remove("declare");
            }
            "TsParenthesizedType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSParenthesizedType".to_string()),
                );
            }
            "TsTypeOperator" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeOperator".to_string()),
                );
            }
            "TsImportType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSImportType".to_string()),
                );
            }
            "TsRestType" => {
                obj.insert("type".to_string(), Value::String("TSRestType".to_string()));
            }
            "TsOptionalType" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSOptionalType".to_string()),
                );
            }
            "TsInferType" => {
                obj.insert("type".to_string(), Value::String("TSInferType".to_string()));
            }
            "TsTypeParameterDeclaration" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeParameterDeclaration".to_string()),
                );
            }
            "TsTypeParameterInstantiation" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeParameterInstantiation".to_string()),
                );
            }
            "TsTypeParameter" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSTypeParameter".to_string()),
                );
            }
            "TsExpressionWithTypeArguments" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSExpressionWithTypeArguments".to_string()),
                );
            }
            "TsSatisfiesExpression" => {
                obj.insert(
                    "type".to_string(),
                    Value::String("TSSatisfiesExpression".to_string()),
                );
            }

            // --- Module types ---
            "Module" => {
                obj.insert("type".to_string(), Value::String("Program".to_string()));
                obj.insert(
                    "sourceType".to_string(),
                    Value::String("module".to_string()),
                );
                obj.remove("interpreter");
            }
            "Script" => {
                obj.insert("type".to_string(), Value::String("Program".to_string()));
                obj.insert(
                    "sourceType".to_string(),
                    Value::String("script".to_string()),
                );
                obj.remove("interpreter");
            }

            _ => {}
        }
    }

    // 4. Remove SWC-specific fields that ESTree doesn't have
    obj.remove("typeOnly");
    obj.remove("definite");
    obj.remove("declare");
    obj.remove("isAbstract");

    // 5. Remove null optional fields common across many node types
    remove_if_null(&mut obj, "returnType");
    remove_if_null(&mut obj, "typeParameters");
    remove_if_null(&mut obj, "typeParams");
    remove_if_null(&mut obj, "superTypeParams");

    // 6. Recursively transform all remaining values
    let mut result: Map<String, Value> = obj
        .into_iter()
        .map(|(k, v)| (k, transform_value(v)))
        .collect();

    // 7. Post-transform fixes that require recursed children
    // Extend Identifier span to include typeAnnotation
    if result.get("type").and_then(|v| v.as_str()) == Some("Identifier") {
        if let Some(Value::Object(ta)) = result.get("typeAnnotation") {
            if let Some(ta_end) = ta.get("end").and_then(|e| e.as_u64()) {
                result.insert("end".to_string(), Value::Number(ta_end.into()));
            }
        }
    }

    // Add missing fields for ArrowFunctionExpression
    if result.get("type").and_then(|v| v.as_str()) == Some("ArrowFunctionExpression") {
        result.entry("id".to_string()).or_insert(Value::Null);
        // expression: true if body is not a BlockStatement
        let is_block = result
            .get("body")
            .and_then(|b| b.as_object())
            .and_then(|b| b.get("type"))
            .and_then(|t| t.as_str())
            == Some("BlockStatement");
        result
            .entry("expression".to_string())
            .or_insert(Value::Bool(!is_block));
    }

    Value::Object(result)
}

/// Remove a field from the map if its value is null.
fn remove_if_null(obj: &mut Map<String, Value>, key: &str) {
    if obj.get(key) == Some(&Value::Null) {
        obj.remove(key);
    }
}

/// Unwrap ExprOrSpread array: [{expr/expression, spread}] → [expr] or [SpreadElement]
fn unwrap_expr_or_spread(args: Value) -> Value {
    match args {
        Value::Array(arr) => {
            Value::Array(
                arr.into_iter()
                    .map(|item| {
                        if let Value::Object(mut obj) = item {
                            // ExprOrSpread has expr/expression + spread fields, no type field
                            if obj.contains_key("spread") && !obj.contains_key("type") {
                                let spread = obj.remove("spread");
                                let expr = obj.remove("expr").or_else(|| obj.remove("expression"));
                                if let Some(expr_val) = expr {
                                    if spread.is_some() && spread != Some(Value::Null) {
                                        // Spread: wrap in SpreadElement
                                        let mut spread_obj = Map::new();
                                        spread_obj.insert(
                                            "type".to_string(),
                                            Value::String("SpreadElement".to_string()),
                                        );
                                        spread_obj.insert("argument".to_string(), expr_val);
                                        Value::Object(spread_obj)
                                    } else {
                                        expr_val
                                    }
                                } else {
                                    Value::Object(obj)
                                }
                            } else {
                                Value::Object(obj)
                            }
                        } else {
                            item
                        }
                    })
                    .collect(),
            )
        }
        other => other,
    }
}
