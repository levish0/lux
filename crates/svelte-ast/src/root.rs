use oxc_ast::ast::{Expression, Program};
use serde::ser::SerializeMap;
use serde::Serialize;

use crate::css::StyleSheet;
use crate::node::{AttributeNode, FragmentNode};
use crate::span::Span;
use crate::text::JsComment;
use crate::utils::estree::OxcSerialize;

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

impl Serialize for Root<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Root")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("fragment", &self.fragment)?;
        map.serialize_entry("options", &self.options)?;
        map.serialize_entry("css", &self.css)?;
        map.serialize_entry("instance", &self.instance)?;
        map.serialize_entry("module", &self.module)?;
        map.serialize_entry("js", &Vec::<()>::new())?;
        map.serialize_entry("comments", &self.comments)?;
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
#[derive(Debug)]
pub struct Script<'a> {
    pub span: Span,
    pub context: ScriptContext,
    pub content: Program<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
}

impl Serialize for Script<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Script")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("context", &self.context)?;
        map.serialize_entry("content", &OxcSerialize(&self.content))?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptContext {
    Default,
    Module,
}

impl Serialize for ScriptContext {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Default => s.serialize_str("default"),
            Self::Module => s.serialize_str("module"),
        }
    }
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

impl Serialize for Fragment<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Fragment")?;
        map.serialize_entry("nodes", &self.nodes)?;
        map.end()
    }
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

impl Serialize for SvelteOptions<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        if let Some(ref runes) = self.runes {
            map.serialize_entry("runes", runes)?;
        }
        if let Some(ref immutable) = self.immutable {
            map.serialize_entry("immutable", immutable)?;
        }
        if let Some(ref accessors) = self.accessors {
            map.serialize_entry("accessors", accessors)?;
        }
        if let Some(ref preserve_whitespace) = self.preserve_whitespace {
            map.serialize_entry("preserveWhitespace", preserve_whitespace)?;
        }
        if let Some(ref namespace) = self.namespace {
            map.serialize_entry("namespace", namespace)?;
        }
        if let Some(ref css) = self.css {
            map.serialize_entry("css", css)?;
        }
        if let Some(ref custom_element) = self.custom_element {
            map.serialize_entry("customElement", custom_element)?;
        }
        map.serialize_entry("attributes", &self.attributes)?;
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Html,
    Svg,
    Mathml,
}

impl Serialize for Namespace {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Html => s.serialize_str("html"),
            Self::Svg => s.serialize_str("svg"),
            Self::Mathml => s.serialize_str("mathml"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssMode {
    Injected,
}

impl Serialize for CssMode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Injected => s.serialize_str("injected"),
        }
    }
}

#[derive(Debug)]
pub struct CustomElementOptions<'a> {
    pub tag: Option<&'a str>,
    pub shadow: Option<ShadowMode>,
    pub props: Option<std::collections::HashMap<&'a str, CustomElementProp<'a>>>,
    pub extend: Option<Expression<'a>>,
}

impl Serialize for CustomElementOptions<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        if let Some(ref tag) = self.tag {
            map.serialize_entry("tag", tag)?;
        }
        if let Some(ref shadow) = self.shadow {
            map.serialize_entry("shadow", shadow)?;
        }
        if let Some(ref props) = self.props {
            map.serialize_entry("props", props)?;
        }
        if let Some(ref extend) = self.extend {
            map.serialize_entry("extend", &OxcSerialize(extend))?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowMode {
    Open,
    None,
}

impl Serialize for ShadowMode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Open => s.serialize_str("open"),
            Self::None => s.serialize_str("none"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomElementProp<'a> {
    pub attribute: Option<&'a str>,
    pub reflect: Option<bool>,
    pub prop_type: Option<PropType>,
}

impl Serialize for CustomElementProp<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        if let Some(ref attribute) = self.attribute {
            map.serialize_entry("attribute", attribute)?;
        }
        if let Some(ref reflect) = self.reflect {
            map.serialize_entry("reflect", reflect)?;
        }
        if let Some(ref prop_type) = self.prop_type {
            map.serialize_entry("type", prop_type)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropType {
    Array,
    Boolean,
    Number,
    Object,
    String,
}

impl Serialize for PropType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Array => s.serialize_str("Array"),
            Self::Boolean => s.serialize_str("Boolean"),
            Self::Number => s.serialize_str("Number"),
            Self::Object => s.serialize_str("Object"),
            Self::String => s.serialize_str("String"),
        }
    }
}
