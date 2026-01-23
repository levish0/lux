use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use swc_ecma_ast as swc;

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
                // Need to include "type": "ExpressionTag" since we're not in a tagged enum
                let mut map = s.serialize_map(None)?;
                map.serialize_entry("type", "ExpressionTag")?;
                map.serialize_entry("start", &e.span.start)?;
                map.serialize_entry("end", &e.span.end)?;
                if e.force_expression_loc {
                    crate::utils::estree::set_force_char_loc(true);
                }
                map.serialize_entry(
                    "expression",
                    &crate::utils::estree::ExprWrapper(&e.expression),
                )?;
                if e.force_expression_loc {
                    crate::utils::estree::set_force_char_loc(false);
                }
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
 *   name_loc: SourceLocation | null;
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SpreadAttribute {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: Option<SourceLocation>,
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
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
    pub expression: Box<swc::Expr>,
    pub leading_comments: Vec<crate::text::JsComment>,
}

impl Serialize for BindDirective {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error as SerError;

        let mut map = s.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;

        // Serialize expression with optional leadingComments
        let expr_value =
            serde_json::to_value(self.expression.as_ref()).map_err(S::Error::custom)?;
        let mut transformed = crate::utils::estree::transform_value_pub(expr_value);
        if !self.leading_comments.is_empty() {
            if let serde_json::Value::Object(ref mut obj) = transformed {
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
        map.serialize_entry("expression", &transformed)?;

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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_expr")]
    pub expression: Option<Box<swc::Expr>>,
    pub modifiers: Vec<EventModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EventModifier {
    Capture,
    Nonpassive,
    Once,
    Passive,
    PreventDefault,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_expr")]
    pub expression: Option<Box<swc::Expr>>,
    pub modifiers: Vec<TransitionModifier>,
    pub intro: bool,
    pub outro: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_expr")]
    pub expression: Option<Box<swc::Expr>>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_expr")]
    pub expression: Option<Box<swc::Expr>>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_expr")]
    pub expression: Option<Box<swc::Expr>>,
}
