use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::JsNode;
use crate::span::{SourceLocation, Span};
use crate::tags::ExpressionTag;
use crate::text::Text;

/*
 * interface Attribute extends BaseNode {
 *   type: 'Attribute';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   value: true | ExpressionTag | Array<Text | ExpressionTag>;
 * }
 */
#[derive(Debug, Clone)]
pub struct Attribute {
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub value: AttributeValue,
}

impl Serialize for Attribute {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    True,
    Expression(ExpressionTag),
    Sequence(Vec<AttributeSequenceValue>),
}

impl Serialize for AttributeValue {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            AttributeValue::True => s.serialize_bool(true),
            AttributeValue::Expression(e) => {
                let mut map = s.serialize_map(None)?;
                map.serialize_entry("type", "ExpressionTag")?;
                map.serialize_entry("start", &e.span.start)?;
                map.serialize_entry("end", &e.span.end)?;
                if e.force_expression_loc {
                    crate::utils::estree::set_force_char_loc(true);
                }
                let mut expr_val = e.expression.0.clone();
                crate::utils::estree::add_loc(&mut expr_val);
                if e.force_expression_loc {
                    crate::utils::estree::set_force_char_loc(false);
                }
                map.serialize_entry("expression", &expr_val)?;
                map.end()
            }
            AttributeValue::Sequence(items) => items.serialize(s),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AttributeSequenceValue {
    Text(Text),
    ExpressionTag(ExpressionTag),
}

/*
 * interface SpreadAttribute extends BaseNode {
 *   type: 'SpreadAttribute';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SpreadAttribute {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JsNode,
}

/*
 * interface BindDirective extends BaseNode {
 *   type: 'BindDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: Identifier | MemberExpression | SequenceExpression;
 * }
 */
#[derive(Debug, Clone)]
pub struct BindDirective {
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: JsNode,
    pub leading_comments: Vec<crate::text::JsComment>,
}

impl Serialize for BindDirective {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;

        // Serialize expression with loc and optional leadingComments
        let mut expr_val = self.expression.0.clone();
        crate::utils::estree::add_loc(&mut expr_val);
        if !self.leading_comments.is_empty() {
            if let serde_json::Value::Object(ref mut obj) = expr_val {
                let comments_val: Vec<serde_json::Value> = self
                    .leading_comments
                    .iter()
                    .map(|c| serde_json::to_value(c).unwrap())
                    .collect();
                obj.insert(
                    "leadingComments".to_string(),
                    serde_json::Value::Array(comments_val),
                );
            }
        }
        map.serialize_entry("expression", &expr_val)?;

        map.serialize_entry("modifiers", &Vec::<String>::new())?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("name_loc", &self.name_loc)?;
        map.end()
    }
}

/*
 * interface ClassDirective extends BaseNode {
 *   type: 'ClassDirective';
 *   name: 'class';
 *   name_loc: SourceLocation | null;
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ClassDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: JsNode,
    pub modifiers: Vec<String>,
}

/*
 * interface StyleDirective extends BaseNode {
 *   type: 'StyleDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   value: true | ExpressionTag | Array<ExpressionTag | Text>;
 *   modifiers: Array<'important'>;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct StyleDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub value: AttributeValue,
    pub modifiers: Vec<StyleModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StyleModifier {
    Important,
}

/*
 * interface OnDirective extends BaseNode {
 *   type: 'OnDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 *   modifiers: string[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct OnDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<JsNode>,
    pub modifiers: Vec<EventModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventModifier {
    Capture,
    Nonpassive,
    Once,
    Passive,
    PreventDefault,
    #[serde(rename = "self")]
    Self_,
    StopImmediatePropagation,
    StopPropagation,
    Trusted,
}

/*
 * interface TransitionDirective extends BaseNode {
 *   type: 'TransitionDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 *   modifiers: Array<'local' | 'global'>;
 *   intro: boolean;
 *   outro: boolean;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct TransitionDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<JsNode>,
    pub modifiers: Vec<TransitionModifier>,
    pub intro: bool,
    pub outro: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransitionModifier {
    Local,
    Global,
}

/*
 * interface AnimateDirective extends BaseNode {
 *   type: 'AnimateDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct AnimateDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<JsNode>,
    pub modifiers: Vec<String>,
}

/*
 * interface UseDirective extends BaseNode {
 *   type: 'UseDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct UseDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<JsNode>,
    pub modifiers: Vec<String>,
}

/*
 * interface LetDirective extends BaseNode {
 *   type: 'LetDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: null | Identifier | ArrayExpression | ObjectExpression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct LetDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<JsNode>,
    pub modifiers: Vec<String>,
}
