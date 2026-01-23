use serde::ser::Error as SerError;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use swc_ecma_ast as swc;

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
pub fn serialize_program<S: Serializer>(
    program: &swc::Program,
    s: S,
) -> Result<S::Ok, S::Error> {
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
    pub content_end: usize,
}

impl<'a> Serialize for ProgramWithComments<'a> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(self.program).map_err(S::Error::custom)?;
        let mut transformed = transform_value(value);

        // Fix Program.end for empty body: extend to content end
        if let Value::Object(ref mut obj) = transformed {
            let body_is_empty = obj
                .get("body")
                .and_then(|b| b.as_array())
                .map(|arr| arr.is_empty())
                .unwrap_or(true);
            if body_is_empty && self.content_end > 0 {
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
                    let comments_val: Vec<Value> = self.comments.iter()
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
            "Identifier" => {
                if let Some(v) = obj.remove("value") {
                    obj.insert("name".to_string(), v);
                }
                obj.remove("optional");
            }
            "CallExpression" => {
                if let Some(args) = obj.remove("args") {
                    obj.insert("arguments".to_string(), args);
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
            "ConditionalExpression" => {
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
            "VariableDeclaration" => {
                if let Some(decls) = obj.remove("decls") {
                    obj.insert("declarations".to_string(), decls);
                }
            }
            "VariableDeclarator" => {
                if let Some(name) = obj.remove("name") {
                    obj.insert("id".to_string(), name);
                }
            }
            "Module" => {
                obj.insert("type".to_string(), Value::String("Program".to_string()));
                obj.insert("sourceType".to_string(), Value::String("module".to_string()));
                obj.remove("interpreter");
            }
            "Script" => {
                obj.insert("type".to_string(), Value::String("Program".to_string()));
                obj.insert("sourceType".to_string(), Value::String("script".to_string()));
                obj.remove("interpreter");
            }
            _ => {}
        }
    }

    // 4. Remove SWC-specific fields that ESTree doesn't have
    obj.remove("typeOnly");
    obj.remove("definite");

    // 5. Recursively transform all remaining values
    let result: Map<String, Value> = obj
        .into_iter()
        .map(|(k, v)| (k, transform_value(v)))
        .collect();

    Value::Object(result)
}
