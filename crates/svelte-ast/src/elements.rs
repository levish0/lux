use serde::Serialize;
use swc_ecma_ast as swc;

use crate::attributes::Attribute;
use crate::node::AttributeNode;
use crate::root::Fragment;
use crate::span::Span;

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
    pub span: Span,
    pub name: String,
    pub name_loc: Span,
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
    pub span: Span,
    pub name: String,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
    pub tag: Box<swc::Expr>,
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
    pub span: Span,
    pub name_loc: Span,
    pub expression: Box<swc::Expr>,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name: String,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
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
    pub span: Span,
    pub name_loc: Span,
    pub attributes: Vec<Attribute>,
}
