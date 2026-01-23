use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use swc_ecma_ast as swc;

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
    pub expression: Box<swc::Expr>,
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
        let expr_val = serde_json::to_value(self.expression.as_ref())
            .map_err(serde::ser::Error::custom)?;
        let expr_transformed = crate::utils::estree::transform_value_pub(expr_val);
        map.serialize_entry("expression", &expr_transformed)?;
        if self.force_expression_loc {
            crate::utils::estree::set_force_char_loc(false);
        }

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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_var_decl")]
    pub declaration: Box<swc::VarDecl>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_idents")]
    pub identifiers: Vec<swc::Ident>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
}
