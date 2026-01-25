use std::cell::RefCell;

use oxc_estree::{CompactTSSerializer, ESTree};

/// Pre-computed line start byte offsets for fast line/column lookup via binary search.
struct LocSource {
    /// Byte offsets where each line starts. line_starts[0] = 0 (line 1 starts at byte 0).
    line_starts: Vec<u32>,
}

impl LocSource {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self { line_starts }
    }

    /// Compute (line, column) from a byte offset using binary search.
    /// Lines are 1-indexed, columns are 0-indexed (ESTree convention).
    fn offset_to_line_col(&self, offset: u32) -> (u32, u32) {
        let line_idx = self
            .line_starts
            .binary_search(&offset)
            .unwrap_or_else(|i| i - 1);
        let line = (line_idx + 1) as u32;
        let col = offset - self.line_starts[line_idx];
        (line, col)
    }
}

thread_local! {
    static LOC_SOURCE: RefCell<Option<LocSource>> = RefCell::new(None);
}

/// Store source text for use during serialization (for computing loc).
pub fn set_loc_source(source: &str) {
    LOC_SOURCE.with(|cell| {
        *cell.borrow_mut() = Some(LocSource::new(source));
    });
}

/// Clear the stored source text.
pub fn clear_loc_source() {
    LOC_SOURCE.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Unwrap ParenthesizedExpression nodes, replacing them with their inner expression.
/// Svelte's AST doesn't preserve parenthesization as node wrappers.
fn unwrap_parenthesized(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            // First recurse into all children
            for (_, v) in map.iter_mut() {
                unwrap_parenthesized(v);
            }
            // Then check if this is a ParenthesizedExpression and unwrap
            if map.get("type").and_then(|v| v.as_str()) == Some("ParenthesizedExpression") {
                if let Some(expr) = map.remove("expression") {
                    *value = expr;
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                unwrap_parenthesized(v);
            }
        }
        _ => {}
    }
}

/// Add `loc` fields to a JSON Value tree. Any object with numeric `start`/`end`
/// fields gets a `loc` field with line/column positions.
fn add_loc(value: &mut serde_json::Value, loc_source: &LocSource) {
    match value {
        serde_json::Value::Object(map) => {
            let has_start_end = matches!(
                (map.get("start"), map.get("end")),
                (
                    Some(serde_json::Value::Number(_)),
                    Some(serde_json::Value::Number(_))
                )
            );
            if has_start_end {
                let start = map["start"].as_u64().unwrap_or(0) as u32;
                let end = map["end"].as_u64().unwrap_or(0) as u32;
                let (start_line, start_col) = loc_source.offset_to_line_col(start);
                let (end_line, end_col) = loc_source.offset_to_line_col(end);
                let loc = serde_json::json!({
                    "start": { "line": start_line, "column": start_col },
                    "end": { "line": end_line, "column": end_col }
                });
                map.insert("loc".to_string(), loc);
            }
            for (_, v) in map.iter_mut() {
                add_loc(v, loc_source);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                add_loc(v, loc_source);
            }
        }
        _ => {}
    }
}

/// Serialize an OXC ESTree node to a `serde_json::Value`, adding `loc` fields,
/// unwrapping ParenthesizedExpression nodes, and fixing regex literal values.
pub fn oxc_node_to_value<T: ESTree>(node: &T) -> serde_json::Value {
    let mut serializer = CompactTSSerializer::default();
    node.serialize(&mut serializer);
    let json_str = serializer.into_string();
    let mut value: serde_json::Value =
        serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);
    unwrap_parenthesized(&mut value);
    LOC_SOURCE.with(|cell| {
        if let Some(ref loc_source) = *cell.borrow() {
            add_loc(&mut value, loc_source);
        }
    });
    value
}

/// Wrapper to serialize an OXC ESTree type via serde.
pub struct OxcSerialize<'a, T: ESTree>(pub &'a T);

impl<T: ESTree> serde::Serialize for OxcSerialize<'_, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = oxc_node_to_value(self.0);
        value.serialize(serializer)
    }
}

/// Wrapper to serialize a Vec of OXC ESTree types via serde.
pub struct OxcVecSerialize<'a, T: ESTree>(pub &'a Vec<T>);

impl<T: ESTree> serde::Serialize for OxcVecSerialize<'_, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            let value = oxc_node_to_value(item);
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

/// Helper to serialize an Option<OXC type>.
pub struct OxcOptionSerialize<'a, T: ESTree>(pub &'a Option<T>);

impl<T: ESTree> serde::Serialize for OxcOptionSerialize<'_, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            Some(node) => {
                let value = oxc_node_to_value(node);
                value.serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }
}
