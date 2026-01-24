use std::cell::RefCell;

use line_span::LineSpanExt;
use serde_json::Value;

thread_local! {
    static LOC_LINE_STARTS: RefCell<Option<Vec<usize>>> = RefCell::new(None);
    static LOC_PATTERN_COLUMN_ADJUST: RefCell<bool> = RefCell::new(false);
    static FORCE_CHAR_LOC: RefCell<bool> = RefCell::new(false);
}

/// Set the source for loc computation. Call before serializing AST nodes.
pub fn set_loc_source(source: &str) {
    let line_starts: Vec<usize> = source.line_spans().map(|s| s.range().start).collect();
    LOC_LINE_STARTS.with(|ls| *ls.borrow_mut() = Some(line_starts));
}

/// Clear the loc source after serialization.
pub fn clear_loc_source() {
    LOC_LINE_STARTS.with(|ls| *ls.borrow_mut() = None);
}

/// Enable pattern column adjustment (+1 for lines > 1).
pub fn set_pattern_column_adjust(v: bool) {
    LOC_PATTERN_COLUMN_ADJUST.with(|f| *f.borrow_mut() = v);
}

/// Set force character-inclusive loc for empty-name Identifiers.
pub fn set_force_char_loc(v: bool) {
    FORCE_CHAR_LOC.with(|f| *f.borrow_mut() = v);
}

fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (usize, usize) {
    let line_idx = line_starts.binary_search(&offset).unwrap_or_else(|idx| idx.saturating_sub(1));
    let line = line_idx + 1;
    let col = offset - line_starts[line_idx];
    let adjust = if line > 1 {
        LOC_PATTERN_COLUMN_ADJUST.with(|f| if *f.borrow() { 1 } else { 0 })
    } else {
        0
    };
    (line, col + adjust)
}

/// Recursively add `offset` to all start/end fields in ESTree JSON.
pub fn adjust_offsets(value: &mut Value, offset: u32) {
    match value {
        Value::Object(obj) => {
            if let Some(Value::Number(n)) = obj.get_mut("start") {
                if let Some(v) = n.as_u64() {
                    *n = (v + offset as u64).into();
                }
            }
            if let Some(Value::Number(n)) = obj.get_mut("end") {
                if let Some(v) = n.as_u64() {
                    *n = (v + offset as u64).into();
                }
            }
            for (_, v) in obj.iter_mut() {
                adjust_offsets(v, offset);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                adjust_offsets(item, offset);
            }
        }
        _ => {}
    }
}

/// Recursively add `loc` fields to all nodes that have start/end.
/// Uses the thread-local LOC_LINE_STARTS source.
pub fn add_loc(value: &mut Value) {
    LOC_LINE_STARTS.with(|ls| {
        if let Some(ref line_starts) = *ls.borrow() {
            add_loc_recursive(value, line_starts);
        }
    });
}

fn add_loc_recursive(value: &mut Value, line_starts: &[usize]) {
    match value {
        Value::Object(obj) => {
            // First recurse into children
            let keys: Vec<String> = obj.keys().cloned().collect();
            for key in keys {
                if let Some(v) = obj.get_mut(&key) {
                    add_loc_recursive(v, line_starts);
                }
            }

            // Then compute loc for this node
            if let (Some(start), Some(end)) = (
                obj.get("start").and_then(|v| v.as_u64()),
                obj.get("end").and_then(|v| v.as_u64()),
            ) {
                let is_empty_ident = obj.get("type").and_then(|v| v.as_str()) == Some("Identifier")
                    && obj.get("name").and_then(|v| v.as_str()) == Some("");

                if is_empty_ident {
                    if start == end && FORCE_CHAR_LOC.with(|f| *f.borrow()) {
                        let (sl, sc) = offset_to_line_col(start as usize, line_starts);
                        let loc = serde_json::json!({
                            "start": {"line": sl, "column": sc, "character": start},
                            "end": {"line": sl, "column": sc, "character": end}
                        });
                        obj.insert("loc".to_string(), loc);
                    }
                } else {
                    let (sl, sc) = offset_to_line_col(start as usize, line_starts);
                    let (el, ec) = offset_to_line_col(end as usize, line_starts);
                    let loc = serde_json::json!({
                        "start": {"line": sl, "column": sc},
                        "end": {"line": el, "column": ec}
                    });
                    obj.insert("loc".to_string(), loc);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                add_loc_recursive(item, line_starts);
            }
        }
        _ => {}
    }
}

/// Add character-inclusive loc to an Identifier node JSON value.
/// Used for name_loc-style Identifiers (snippet expression, each/await patterns).
pub fn add_char_loc(value: &mut Value) {
    LOC_LINE_STARTS.with(|ls| {
        if let Some(ref line_starts) = *ls.borrow() {
            if let Value::Object(obj) = value {
                if let (Some(start), Some(end)) = (
                    obj.get("start").and_then(|v| v.as_u64()),
                    obj.get("end").and_then(|v| v.as_u64()),
                ) {
                    let (sl, sc) = offset_to_line_col(start as usize, line_starts);
                    let (el, ec) = offset_to_line_col(end as usize, line_starts);
                    let loc = serde_json::json!({
                        "start": {"line": sl, "column": sc, "character": start},
                        "end": {"line": el, "column": ec, "character": end}
                    });
                    obj.insert("loc".to_string(), loc);
                }
            }
        }
    });
}

/// Add loc to a Program node using script tag boundaries for loc,
/// and content boundaries for start/end.
pub fn add_program_loc(
    value: &mut Value,
    content_start: usize,
    content_end: usize,
    script_start: usize,
    script_end: usize,
) {
    if let Value::Object(obj) = value {
        obj.insert("start".to_string(), Value::Number(content_start.into()));
        obj.insert("end".to_string(), Value::Number(content_end.into()));

        LOC_LINE_STARTS.with(|ls| {
            if let Some(ref line_starts) = *ls.borrow() {
                let (sl, sc) = offset_to_line_col(script_start, line_starts);
                let (el, ec) = offset_to_line_col(script_end, line_starts);
                let loc = serde_json::json!({
                    "start": {"line": sl, "column": sc},
                    "end": {"line": el, "column": ec}
                });
                obj.insert("loc".to_string(), loc);
            }
        });
    }
}
