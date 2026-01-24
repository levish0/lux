use std::cell::RefCell;

use line_span::LineSpanExt;
use serde_json::Value;

thread_local! {
    static LOC_LINE_STARTS: RefCell<Option<Vec<usize>>> = RefCell::new(None);
    static LOC_SOURCE: RefCell<Option<String>> = RefCell::new(None);
    static LOC_PATTERN_COLUMN_ADJUST: RefCell<bool> = RefCell::new(false);
    static FORCE_CHAR_LOC: RefCell<bool> = RefCell::new(false);
}

/// Set the source for loc computation. Call before serializing AST nodes.
pub fn set_loc_source(source: &str) {
    let line_starts: Vec<usize> = source.line_spans().map(|s| s.range().start).collect();
    LOC_LINE_STARTS.with(|ls| *ls.borrow_mut() = Some(line_starts));
    LOC_SOURCE.with(|s| *s.borrow_mut() = Some(source.to_string()));
}

/// Clear the loc source after serialization.
pub fn clear_loc_source() {
    LOC_LINE_STARTS.with(|ls| *ls.borrow_mut() = None);
    LOC_SOURCE.with(|s| *s.borrow_mut() = None);
}

/// Get the stored source text.
pub fn get_loc_source() -> Option<String> {
    LOC_SOURCE.with(|s| s.borrow().clone())
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

/// Attach comments to individual AST nodes following Svelte's reference algorithm.
/// Comments are consumed in DFS order and attached as leadingComments/trailingComments.
pub fn attach_comments(program: &mut Value, comments: &[crate::text::JsComment], source: &str) {
    if comments.is_empty() {
        return;
    }

    // Convert comments to JSON values (type, value, start, end - no loc)
    let mut comment_vals: Vec<Value> = comments
        .iter()
        .map(|c| {
            let type_str = match c.kind {
                crate::text::JsCommentKind::Line => "Line",
                crate::text::JsCommentKind::Block => "Block",
            };
            serde_json::json!({
                "type": type_str,
                "value": c.value,
                "start": c.span.map_or(0, |s| s.start),
                "end": c.span.map_or(0, |s| s.end),
            })
        })
        .collect();

    // Walk tree and attach
    walk_attach(program, &mut comment_vals, source);

    // Remaining comments become Program's trailingComments
    if !comment_vals.is_empty() {
        if let Value::Object(obj) = program {
            let trailing = obj
                .entry("trailingComments")
                .or_insert(Value::Array(vec![]));
            if let Value::Array(arr) = trailing {
                arr.extend(comment_vals.drain(..));
            }
        }
    }
}

fn get_start(v: &Value) -> u64 {
    v.get("start").and_then(|s| s.as_u64()).unwrap_or(0)
}

fn get_end(v: &Value) -> u64 {
    v.get("end").and_then(|s| s.as_u64()).unwrap_or(0)
}

fn walk_attach(node: &mut Value, comments: &mut Vec<Value>, source: &str) {
    if comments.is_empty() {
        return;
    }

    if let Value::Object(obj) = node {
        let node_start = obj.get("start").and_then(|v| v.as_u64()).unwrap_or(0);
        let node_end = obj.get("end").and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
        let node_type = obj
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        // Consume comments before this node's start as its leadingComments
        {
            let mut leading = Vec::new();
            while !comments.is_empty() && get_start(&comments[0]) < node_start {
                leading.push(comments.remove(0));
            }
            if !leading.is_empty() {
                obj.insert("leadingComments".to_string(), Value::Array(leading));
            }
        }

        // Process body-like arrays (Program.body, BlockStatement.body, SwitchCase.consequent)
        let body_keys: Vec<&str> = match node_type.as_str() {
            "Program" | "BlockStatement" | "StaticBlock" | "ClassBody" => vec!["body"],
            "SwitchCase" => vec!["consequent"],
            _ => vec![],
        };

        for body_key in &body_keys {
            if let Some(Value::Array(children)) = obj.get_mut(*body_key) {
                let len = children.len();
                for i in 0..len {
                    if comments.is_empty() {
                        break;
                    }

                    let child_start = get_start(&children[i]);

                    // Leading: consume comments before child's start
                    let mut leading = Vec::new();
                    while !comments.is_empty() && get_start(&comments[0]) < child_start {
                        leading.push(comments.remove(0));
                    }
                    if !leading.is_empty() {
                        if let Value::Object(child_obj) = &mut children[i] {
                            child_obj
                                .insert("leadingComments".to_string(), Value::Array(leading));
                        }
                    }

                    // Recurse into child
                    walk_attach(&mut children[i], comments, source);

                    // Trailing
                    if !comments.is_empty() {
                        let child_end = get_end(&children[i]);
                        let is_last = i == len - 1;

                        if is_last {
                            // Last in body: consume all remaining within parent's scope
                            let mut trailing = Vec::new();
                            while !comments.is_empty() {
                                if get_start(&comments[0]) >= node_end {
                                    break;
                                }
                                trailing.push(comments.remove(0));
                            }
                            if !trailing.is_empty() {
                                if let Value::Object(child_obj) = &mut children[i] {
                                    child_obj.insert(
                                        "trailingComments".to_string(),
                                        Value::Array(trailing),
                                    );
                                }
                            }
                        } else {
                            // Not last: single trailing if only whitespace/punctuation between
                            let c_start = get_start(&comments[0]);
                            if child_end <= c_start && (c_start as usize) <= source.len() {
                                let end_idx = child_end as usize;
                                let start_idx = c_start as usize;
                                if end_idx <= source.len() && start_idx <= source.len() {
                                    let slice = &source[end_idx..start_idx];
                                    if slice.chars().all(|c| matches!(c, ',' | ')' | ' ' | '\t')) {
                                        let comment = comments.remove(0);
                                        if let Value::Object(child_obj) = &mut children[i] {
                                            child_obj.insert(
                                                "trailingComments".to_string(),
                                                Value::Array(vec![comment]),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Recurse into other fields that may contain nested bodies
        let keys: Vec<String> = obj.keys().cloned().collect();
        for key in keys {
            if body_keys.contains(&key.as_str()) {
                continue; // already handled
            }
            if let Some(v) = obj.get_mut(&key) {
                match v {
                    Value::Object(_) => walk_attach(v, comments, source),
                    Value::Array(arr) => {
                        for item in arr.iter_mut() {
                            if item.is_object() {
                                walk_attach(item, comments, source);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
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
