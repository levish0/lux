use serde::Serialize;

/*
 * interface Text extends BaseNode {
 *   type: 'Text';
 *   data: string;
 *   raw: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Text {
    pub data: String,
    pub raw: String,
}

/*
 * interface Comment extends BaseNode {
 *   type: 'Comment';
 *   data: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Comment {
    pub data: String,
}
