use oxc_ast::ast::Expression;
use serde::ser::SerializeMap;
use serde::Serialize;

use crate::span::{SourceLocation, Span};
use crate::tags::ExpressionTag;
use crate::text::Text;
use crate::utils::estree::{OxcOptionSerialize, OxcSerialize};

/*
 * interface Attribute extends BaseAttribute {
 *   type: 'Attribute';
 *   value: true | ExpressionTag | Array<Text | ExpressionTag>;
 * }
 */
#[derive(Debug)]
pub struct Attribute<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub value: AttributeValue<'a>,
}

impl Serialize for Attribute<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Attribute")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    True,
    ExpressionTag(ExpressionTag<'a>),
    Sequence(Vec<AttributeSequenceValue<'a>>),
}

impl Serialize for AttributeValue<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::True => s.serialize_bool(true),
            Self::ExpressionTag(tag) => {
                use serde::ser::SerializeSeq;
                let mut seq = s.serialize_seq(Some(1))?;
                seq.serialize_element(tag)?;
                seq.end()
            }
            Self::Sequence(items) => items.serialize(s),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AttributeSequenceValue<'a> {
    Text(Text<'a>),
    ExpressionTag(ExpressionTag<'a>),
}

/*
 * interface SpreadAttribute extends BaseNode {
 *   type: 'SpreadAttribute';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct SpreadAttribute<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

impl Serialize for SpreadAttribute<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "SpreadAttribute")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.end()
    }
}

/*
 * interface BindDirective extends BaseAttribute {
 *   type: 'BindDirective';
 *   name: string;
 *   expression: Identifier | MemberExpression | SequenceExpression;
 * }
 */
#[derive(Debug)]
pub struct BindDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub expression: Expression<'a>,
    pub modifiers: Vec<&'a str>,
}

impl Serialize for BindDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "BindDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.end()
    }
}

/*
 * interface ClassDirective extends BaseAttribute {
 *   type: 'ClassDirective';
 *   name: 'class';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct ClassDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub expression: Expression<'a>,
    pub modifiers: Vec<&'a str>,
}

impl Serialize for ClassDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "ClassDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.end()
    }
}

/*
 * interface StyleDirective extends BaseAttribute {
 *   type: 'StyleDirective';
 *   name: string;
 *   value: true | ExpressionTag | Array<ExpressionTag | Text>;
 *   modifiers: Array<'important'>;
 * }
 */
#[derive(Debug)]
pub struct StyleDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub value: AttributeValue<'a>,
    pub modifiers: Vec<&'a str>,
}

impl Serialize for StyleDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "StyleDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.end()
    }
}

/*
 * interface OnDirective extends BaseAttribute {
 *   type: 'OnDirective';
 *   name: string;
 *   expression: null | Expression;
 *   modifiers: string[];
 * }
 */
#[derive(Debug)]
pub struct OnDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<&'a str>,
}

impl Serialize for OnDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "OnDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcOptionSerialize(&self.expression))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.end()
    }
}

/*
 * interface TransitionDirective extends BaseAttribute {
 *   type: 'TransitionDirective';
 *   name: string;
 *   expression: null | Expression;
 *   modifiers: Array<'local' | 'global'>;
 *   intro: boolean;
 *   outro: boolean;
 * }
 */
#[derive(Debug)]
pub struct TransitionDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<&'a str>,
    pub intro: bool,
    pub outro: bool,
}

impl Serialize for TransitionDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "TransitionDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcOptionSerialize(&self.expression))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.serialize_entry("intro", &self.intro)?;
        map.serialize_entry("outro", &self.outro)?;
        map.end()
    }
}

/*
 * interface AnimateDirective extends BaseAttribute {
 *   type: 'AnimateDirective';
 *   name: string;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug)]
pub struct AnimateDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<&'a str>,
}

impl Serialize for AnimateDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "AnimateDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcOptionSerialize(&self.expression))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.end()
    }
}

/*
 * interface UseDirective extends BaseAttribute {
 *   type: 'UseDirective';
 *   name: string;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug)]
pub struct UseDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<&'a str>,
}

impl Serialize for UseDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "UseDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcOptionSerialize(&self.expression))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.end()
    }
}

/*
 * interface LetDirective extends BaseAttribute {
 *   type: 'LetDirective';
 *   name: string;
 *   expression: null | Identifier | ArrayExpression | ObjectExpression;
 * }
 */
#[derive(Debug)]
pub struct LetDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<&'a str>,
}

impl Serialize for LetDirective<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "LetDirective")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("expression", &OxcOptionSerialize(&self.expression))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.end()
    }
}
