use serde::Serialize;

use crate::JsNode;
use crate::node::AttributeNode;
use crate::root::Fragment;
use crate::span::{SourceLocation, Span};

/*
 * interface RegularElement extends BaseElement {
 *   type: 'RegularElement';
 *   name: string;
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct RegularElement {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface Component extends BaseElement {
 *   type: 'Component';
 *   name: string;
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Component {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteElement extends BaseElement {
 *   type: 'SvelteElement';
 *   name: 'svelte:element';
 *   tag: Expression;
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteElement {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub tag: JsNode,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteComponent extends BaseElement {
 *   type: 'SvelteComponent';
 *   name: 'svelte:component';
 *   expression: Expression;
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteComponent {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub expression: JsNode,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteSelf extends BaseElement {
 *   type: 'SvelteSelf';
 *   name: 'svelte:self';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteSelf {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SlotElement extends BaseElement {
 *   type: 'SlotElement';
 *   name: 'slot';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SlotElement {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteHead extends BaseElement {
 *   type: 'SvelteHead';
 *   name: 'svelte:head';
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteHead {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub fragment: Fragment,
}

/*
 * interface SvelteBody extends BaseElement {
 *   type: 'SvelteBody';
 *   name: 'svelte:body';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteBody {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteWindow extends BaseElement {
 *   type: 'SvelteWindow';
 *   name: 'svelte:window';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteWindow {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteDocument extends BaseElement {
 *   type: 'SvelteDocument';
 *   name: 'svelte:document';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteDocument {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteFragment extends BaseElement {
 *   type: 'SvelteFragment';
 *   name: 'svelte:fragment';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteFragment {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteBoundary extends BaseElement {
 *   type: 'SvelteBoundary';
 *   name: 'svelte:boundary';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteBoundary {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface TitleElement extends BaseElement {
 *   type: 'TitleElement';
 *   name: 'title';
 *   attributes: Array<Attribute | SpreadAttribute | Directive>;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct TitleElement {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
    pub fragment: Fragment,
}

/*
 * interface SvelteOptionsRaw extends BaseElement {
 *   type: 'SvelteOptions';
 *   name: 'svelte:options';
 *   attributes: Attribute[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SvelteOptionsRaw {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode>,
}
