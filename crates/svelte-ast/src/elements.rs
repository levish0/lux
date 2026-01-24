use oxc_ast::ast::Expression;
use serde::ser::SerializeMap;
use serde::Serialize;

use crate::node::AttributeNode;
use crate::root::Fragment;
use crate::span::{SourceLocation, Span};
use crate::utils::estree::OxcSerialize;

/*
 * interface BaseElement extends BaseNode {
 *   name: string;
 *   name_loc: SourceLocation;
 *   attributes: Array<Attribute | SpreadAttribute | Directive | AttachTag>;
 *   fragment: Fragment;
 * }
 */

macro_rules! impl_element_serialize {
    ($ty:ident, $type_str:expr) => {
        impl Serialize for $ty<'_> {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut map = s.serialize_map(None)?;
                map.serialize_entry("type", $type_str)?;
                map.serialize_entry("start", &self.span.start)?;
                map.serialize_entry("end", &self.span.end)?;
                map.serialize_entry("name", &self.name)?;
                map.serialize_entry("name_loc", &self.name_loc)?;
                map.serialize_entry("attributes", &self.attributes)?;
                map.serialize_entry("fragment", &self.fragment)?;
                map.end()
            }
        }
    };
}

#[derive(Debug)]
pub struct RegularElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(RegularElement, "RegularElement");

#[derive(Debug)]
pub struct Component<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(Component, "Component");

#[derive(Debug)]
pub struct SvelteElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub tag: Expression<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl Serialize for SvelteElement<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "SvelteElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("tag", &OxcSerialize(&self.tag))?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.serialize_entry("fragment", &self.fragment)?;
        map.end()
    }
}

#[derive(Debug)]
pub struct SvelteComponent<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub expression: Expression<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl Serialize for SvelteComponent<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "SvelteComponent")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.serialize_entry("fragment", &self.fragment)?;
        map.end()
    }
}

#[derive(Debug)]
pub struct SvelteSelf<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteSelf, "SvelteSelf");

#[derive(Debug)]
pub struct SlotElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SlotElement, "SlotElement");

#[derive(Debug)]
pub struct SvelteHead<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteHead, "SvelteHead");

#[derive(Debug)]
pub struct SvelteBody<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteBody, "SvelteBody");

#[derive(Debug)]
pub struct SvelteWindow<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteWindow, "SvelteWindow");

#[derive(Debug)]
pub struct SvelteDocument<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteDocument, "SvelteDocument");

#[derive(Debug)]
pub struct SvelteFragment<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteFragment, "SvelteFragment");

#[derive(Debug)]
pub struct SvelteBoundary<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteBoundary, "SvelteBoundary");

#[derive(Debug)]
pub struct TitleElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(TitleElement, "TitleElement");

#[derive(Debug)]
pub struct SvelteOptionsRaw<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

impl_element_serialize!(SvelteOptionsRaw, "SvelteOptionsRaw");
