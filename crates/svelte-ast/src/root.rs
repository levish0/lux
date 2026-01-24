use oxc_ast::ast::{Expression, Program};

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
#[derive(Debug)]
pub struct Root<'a> {
    pub span: Span,
    pub options: Option<SvelteOptions<'a>>,
    pub fragment: Fragment<'a>,
    pub css: Option<StyleSheet<'a>>,
    pub instance: Option<Script<'a>>,
    pub module: Option<Script<'a>>,
    pub comments: Vec<JsComment<'a>>,
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
#[derive(Debug)]
pub struct Script<'a> {
    pub span: Span,
    pub context: ScriptContext,
    pub content: Program<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug)]
pub struct Fragment<'a> {
    pub nodes: Vec<FragmentNode<'a>>,
}

/*
 * interface SvelteOptions {
 *   start: number;
 *   end: number;
 *   runes?: boolean;
 *   immutable?: boolean;
 *   accessors?: boolean;
 *   preserveWhitespace?: boolean;
 *   namespace?: Namespace;
 *   css?: 'injected';
 *   customElement?: { ... };
 *   attributes: Attribute[];
 * }
 */
#[derive(Debug)]
pub struct SvelteOptions<'a> {
    pub span: Span,
    pub runes: Option<bool>,
    pub immutable: Option<bool>,
    pub accessors: Option<bool>,
    pub preserve_whitespace: Option<bool>,
    pub namespace: Option<Namespace>,
    pub css: Option<CssMode>,
    pub custom_element: Option<CustomElementOptions<'a>>,
    pub attributes: Vec<AttributeNode<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Html,
    Svg,
    Mathml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssMode {
    Injected,
}

#[derive(Debug)]
pub struct CustomElementOptions<'a> {
    pub tag: Option<&'a str>,
    pub shadow: Option<ShadowMode>,
    pub props: Option<std::collections::HashMap<&'a str, CustomElementProp<'a>>>,
    pub extend: Option<Expression<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowMode {
    Open,
    None,
}

#[derive(Debug, Clone)]
pub struct CustomElementProp<'a> {
    pub attribute: Option<&'a str>,
    pub reflect: Option<bool>,
    pub prop_type: Option<PropType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropType {
    Array,
    Boolean,
    Number,
    Object,
    String,
}
