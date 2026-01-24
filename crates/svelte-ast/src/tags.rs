use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::JsNode;
use crate::span::Span;

/*
 * interface ExpressionTag extends BaseNode {
 *   type: 'ExpressionTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone)]
pub struct ExpressionTag {
    pub span: Span,
    pub expression: JsNode,
    /// When true, expression gets character-inclusive loc (shorthand attribute case)
    pub force_expression_loc: bool,
}

impl Serialize for ExpressionTag {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", "ExpressionTag")?;

        if self.force_expression_loc {
            crate::utils::estree::set_force_char_loc(true);
        }
        let mut expr_val = self.expression.0.clone();
        crate::utils::estree::add_loc(&mut expr_val);
        if self.force_expression_loc {
            crate::utils::estree::set_force_char_loc(false);
        }
        map.serialize_entry("expression", &expr_val)?;

        map.end()
    }
}

/*
 * interface HtmlTag extends BaseNode {
 *   type: 'HtmlTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct HtmlTag {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JsNode,
}

/*
 * interface ConstTag extends BaseNode {
 *   type: 'ConstTag';
 *   declaration: VariableDeclaration;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ConstTag {
    #[serde(flatten)]
    pub span: Span,
    pub declaration: JsNode,
}

/*
 * interface DebugTag extends BaseNode {
 *   type: 'DebugTag';
 *   identifiers: Identifier[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct DebugTag {
    #[serde(flatten)]
    pub span: Span,
    pub identifiers: JsNode,
}

/*
 * interface RenderTag extends BaseNode {
 *   type: 'RenderTag';
 *   expression: SimpleCallExpression | (ChainExpression & { expression: SimpleCallExpression });
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct RenderTag {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JsNode,
}

/*
 * interface AttachTag extends BaseNode {
 *   type: 'AttachTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct AttachTag {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JsNode,
}
