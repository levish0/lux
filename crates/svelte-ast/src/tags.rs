use oxc_ast::ast::{Expression, VariableDeclaration};
use serde::ser::SerializeMap;
use serde::Serialize;

use crate::metadata::{ExpressionNodeMetadata, RenderTagMetadata};
use crate::span::Span;
use crate::utils::estree::{OxcSerialize, OxcVecSerialize};

/*
 * interface ExpressionTag extends BaseNode {
 *   type: 'ExpressionTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct ExpressionTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub metadata: ExpressionNodeMetadata,
}

impl Serialize for ExpressionTag<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "ExpressionTag")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.end()
    }
}

/*
 * interface HtmlTag extends BaseNode {
 *   type: 'HtmlTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct HtmlTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub metadata: ExpressionNodeMetadata,
}

impl Serialize for HtmlTag<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "HtmlTag")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.end()
    }
}

/*
 * interface ConstTag extends BaseNode {
 *   type: 'ConstTag';
 *   declaration: VariableDeclaration;
 * }
 */
#[derive(Debug)]
pub struct ConstTag<'a> {
    pub span: Span,
    pub declaration: VariableDeclaration<'a>,
    pub metadata: ExpressionNodeMetadata,
}

impl Serialize for ConstTag<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "ConstTag")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("declaration", &OxcSerialize(&self.declaration))?;
        map.end()
    }
}

/*
 * interface DebugTag extends BaseNode {
 *   type: 'DebugTag';
 *   identifiers: Identifier[];
 * }
 */
#[derive(Debug)]
pub struct DebugTag<'a> {
    pub span: Span,
    pub identifiers: Vec<Expression<'a>>,
}

impl Serialize for DebugTag<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "DebugTag")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("identifiers", &OxcVecSerialize(&self.identifiers))?;
        map.end()
    }
}

/*
 * interface RenderTag extends BaseNode {
 *   type: 'RenderTag';
 *   expression: SimpleCallExpression | (ChainExpression & { expression: SimpleCallExpression });
 * }
 */
#[derive(Debug)]
pub struct RenderTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub metadata: RenderTagMetadata,
}

impl Serialize for RenderTag<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "RenderTag")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.end()
    }
}

/*
 * interface AttachTag extends BaseNode {
 *   type: 'AttachTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct AttachTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

impl Serialize for AttachTag<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "AttachTag")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.end()
    }
}
