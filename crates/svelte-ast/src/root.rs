use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::HashMap;

use crate::JsNode;
use crate::css::StyleSheet;
use crate::node::{AttributeNode, FragmentNode};
use crate::span::Span;
use crate::text::JsComment;

/*
 * interface Root extends BaseNode {
 *   type: 'Root';
 *   options: SvelteOptions | null;
 *   fragment: Fragment;
 *   css: AST.CSS.StyleSheet | null;
 *   instance: Script | null;
 *   module: Script | null;
 *   comments: JSComment[];
 *   metadata: { ts: boolean };
 * }
 */
#[derive(Debug, Clone)]
pub struct Root {
    pub span: Span,
    pub options: Option<SvelteOptions>,
    pub fragment: Fragment,
    pub css: Option<StyleSheet>,
    pub instance: Option<Script>,
    pub module: Option<Script>,
    pub comments: Vec<JsComment>,
    pub ts: bool,
}

impl Serialize for Root {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("css", &self.css)?;
        map.serialize_entry("js", &Vec::<()>::new())?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", "Root")?;
        map.serialize_entry("fragment", &self.fragment)?;
        map.serialize_entry("options", &self.options)?;
        if self.instance.is_some() || self.module.is_some() {
            if self.module.is_some() {
                map.serialize_entry("module", &self.module)?;
            }
            if self.instance.is_some() {
                map.serialize_entry("instance", &self.instance)?;
            }
        }
        map.end()
    }
}

/*
 * interface Script extends BaseNode {
 *   type: 'Script';
 *   context: 'default' | 'module';
 *   content: Program;
 *   attributes: Attribute[];
 * }
 */
#[derive(Debug, Clone)]
pub struct Script {
    pub span: Span,
    pub context: ScriptContext,
    pub content: JsNode,
    pub content_comments: Vec<JsComment>,
    pub content_start: usize,
    pub content_end: usize,
    pub attributes: Vec<AttributeNode>,
}

impl Serialize for Script {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Script")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("context", &self.context)?;

        // Build program value with comments and loc
        let mut program = self.content.0.clone();

        // Set Program start/end to content boundaries, loc to script tag boundaries
        crate::utils::estree::add_program_loc(
            &mut program,
            self.content_start,
            self.content_end,
            self.span.start,
            self.span.end,
        );

        // Attach comments
        if !self.content_comments.is_empty() {
            if let serde_json::Value::Object(ref mut obj) = program {
                let body_is_empty = obj
                    .get("body")
                    .and_then(|b| b.as_array())
                    .map(|arr| arr.is_empty())
                    .unwrap_or(true);

                if body_is_empty {
                    let comments_val: Vec<serde_json::Value> = self
                        .content_comments
                        .iter()
                        .map(|c| serde_json::to_value(c).map_err(serde::ser::Error::custom))
                        .collect::<Result<_, _>>()?;
                    obj.insert("trailingComments".to_string(), serde_json::Value::Array(comments_val));
                } else {
                    let body_start = obj
                        .get("body")
                        .and_then(|b| b.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|node| node.get("start"))
                        .and_then(|s| s.as_u64())
                        .unwrap_or(u64::MAX);

                    let mut leading = Vec::new();
                    let mut trailing = Vec::new();

                    for comment in &self.content_comments {
                        let c_val = serde_json::to_value(comment).map_err(serde::ser::Error::custom)?;
                        let comment_start = comment.span.map_or(0, |s| s.start) as u64;
                        if comment_start < body_start {
                            leading.push(c_val);
                        } else {
                            trailing.push(c_val);
                        }
                    }

                    if !leading.is_empty() {
                        obj.insert("leadingComments".to_string(), serde_json::Value::Array(leading));
                    }
                    if !trailing.is_empty() {
                        obj.insert("trailingComments".to_string(), serde_json::Value::Array(trailing));
                    }
                }
            }
        }

        map.serialize_entry("content", &program)?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptContext {
    Default,
    Module,
}

/*
 * interface Fragment {
 *   type: 'Fragment';
 *   nodes: Array<Text | Tag | ElementLike | Block | Comment>;
 * }
 */
#[derive(Debug, Clone)]
pub struct Fragment {
    pub nodes: Vec<FragmentNode>,
}

impl Serialize for Fragment {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Fragment")?;
        map.serialize_entry("nodes", &self.nodes)?;
        map.end()
    }
}

/*
 * interface SvelteOptions {
 *   runes?: boolean;
 *   immutable?: boolean;
 *   accessors?: boolean;
 *   preserveWhitespace?: boolean;
 *   namespace?: 'html' | 'svg' | 'mathml';
 *   css?: 'injected';
 *   customElement?: { tag?: string; shadow?: 'open' | 'none'; props?: Record<string, ...>; extend?: ArrowFunctionExpression | Identifier; };
 *   attributes: Attribute[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteOptions {
    #[serde(flatten)]
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immutable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessors: Option<bool>,
    #[serde(rename = "preserveWhitespace", skip_serializing_if = "Option::is_none")]
    pub preserve_whitespace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Namespace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<CssMode>,
    #[serde(rename = "customElement", skip_serializing_if = "Option::is_none")]
    pub custom_element: Option<CustomElementOptions>,
    pub attributes: Vec<AttributeNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Namespace {
    Html,
    Svg,
    Mathml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CssMode {
    Injected,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomElementOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<ShadowMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<HashMap<String, CustomElementProp>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extend: Option<JsNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ShadowMode {
    Open,
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomElementProp {
    pub attribute: Option<String>,
    pub reflect: Option<bool>,
    pub prop_type: Option<PropType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PropType {
    Array,
    Boolean,
    Number,
    Object,
    String,
}
