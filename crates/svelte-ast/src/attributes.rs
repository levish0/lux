use oxc_ast::ast::Expression;

use crate::span::{SourceLocation, Span};
use crate::tags::ExpressionTag;
use crate::text::Text;

/*
 * interface Attribute extends BaseAttribute {
 *   type: 'Attribute';
 *   value: true | ExpressionTag | Array<Text | ExpressionTag>;
 * }
 */
#[derive(Debug)]
pub struct Attribute<'a> {
    pub span: Span,
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub value: AttributeValue<'a>,
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    True,
    ExpressionTag(ExpressionTag<'a>),
    Sequence(Vec<AttributeSequenceValue<'a>>),
}

#[derive(Debug)]
pub enum AttributeSequenceValue<'a> {
    Text(Text),
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Expression<'a>,
    pub modifiers: Vec<String>,
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Expression<'a>,
    pub modifiers: Vec<String>,
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub value: AttributeValue<'a>,
    pub modifiers: Vec<String>,
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<String>,
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<String>,
    pub intro: bool,
    pub outro: bool,
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<String>,
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<String>,
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
    pub name: String,
    pub name_loc: Option<SourceLocation>,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<String>,
}
