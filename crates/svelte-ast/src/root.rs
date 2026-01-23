use serde::Serialize;
use std::collections::HashMap;
use swc_ecma_ast as swc;

use crate::attributes::Attribute;
use crate::css::StyleSheet;
use crate::node::FragmentNode;
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
#[derive(Debug, Clone, Serialize)]
pub struct Root {
    #[serde(flatten)]
    pub span: Span,
    pub fragment: Fragment,
    pub css: Option<StyleSheet>,
    pub instance: Option<Script>,
    pub module: Option<Script>,
    pub options: Option<SvelteOptions>,
    pub comments: Vec<JsComment>,
    pub ts: bool,
}

/*
 * interface Script extends BaseNode {
 *   type: 'Script';
 *   context: 'default' | 'module';
 *   content: Program;
 *   attributes: Attribute[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Script {
    #[serde(flatten)]
    pub span: Span,
    pub context: ScriptContext,
    pub content: swc::Program,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
pub struct Fragment {
    pub nodes: Vec<FragmentNode>,
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
    pub runes: Option<bool>,
    pub immutable: Option<bool>,
    pub accessors: Option<bool>,
    pub preserve_whitespace: Option<bool>,
    pub namespace: Option<Namespace>,
    pub css: Option<CssMode>,
    pub custom_element: Option<CustomElementOptions>,
    pub attributes: Vec<Attribute>,
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
    pub tag: Option<String>,
    pub shadow: Option<ShadowMode>,
    pub props: Option<HashMap<String, CustomElementProp>>,
    pub extend: Option<Box<swc::Expr>>,
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
