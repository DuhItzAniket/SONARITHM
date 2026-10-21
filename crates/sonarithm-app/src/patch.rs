// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spec patches (B05-T01): JSON Patch operations (RFC 6902) on the session's spec. A patch
//! applies atomically: if one operation fails, the spec does not change. Every applied
//! patch is one undoable edit on the command bus (FR-111).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// One JSON Patch operation (RFC 6902 §4). Paths are JSON pointers (RFC 6901), the same
/// pointers diagnostics carry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "op", rename_all = "lowercase")]
#[non_exhaustive]
pub enum PatchOp {
    /// Adds a member or inserts an array element (`-` appends).
    Add {
        /// Where.
        path: String,
        /// The value.
        value: Value,
    },
    /// Removes a member or element.
    Remove {
        /// What.
        path: String,
    },
    /// Replaces an existing value.
    Replace {
        /// What.
        path: String,
        /// The new value.
        value: Value,
    },
    /// Moves a value.
    Move {
        /// From where.
        from: String,
        /// To where.
        path: String,
    },
    /// Copies a value.
    Copy {
        /// From where.
        from: String,
        /// To where.
        path: String,
    },
    /// Checks a value; the patch fails if it differs.
    Test {
        /// Where.
        path: String,
        /// The expected value.
        value: Value,
    },
}

/// Applies `ops` to `document`, all or nothing.
///
/// # Errors
///
/// A message naming the failed operation (by position) and why.
pub fn apply(document: &Value, ops: &[PatchOp]) -> Result<Value, String> {
    let mut doc = document.clone();
    for (index, op) in ops.iter().enumerate() {
        apply_one(&mut doc, op).map_err(|why| format!("operation {index} failed: {why}"))?;
    }
    Ok(doc)
}

fn apply_one(doc: &mut Value, op: &PatchOp) -> Result<(), String> {
    match op {
        PatchOp::Add { path, value } => add(doc, path, value.clone()),
        PatchOp::Remove { path } => remove(doc, path).map(|_| ()),
        PatchOp::Replace { path, value } => {
            let target = get_mut(doc, &tokens(path)?).ok_or_else(|| missing(path))?;
            *target = value.clone();
            Ok(())
        }
        PatchOp::Move { from, path } => {
            if path != from
                && path.starts_with(from.as_str())
                && path[from.len()..].starts_with('/')
            {
                return Err(format!("cannot move {from} into its own child {path}"));
            }
            let value = remove(doc, from)?;
            add(doc, path, value)
        }
        PatchOp::Copy { from, path } => {
            let value = get(doc, &tokens(from)?)
                .ok_or_else(|| missing(from))?
                .clone();
            add(doc, path, value)
        }
        PatchOp::Test { path, value } => {
            let found = get(doc, &tokens(path)?).ok_or_else(|| missing(path))?;
            if found == value {
                Ok(())
            } else {
                Err(format!("the value at {path} is not the expected one"))
            }
        }
    }
}

fn missing(path: &str) -> String {
    format!("nothing is at {path}")
}

/// The unescaped tokens of a JSON pointer (RFC 6901).
fn tokens(pointer: &str) -> Result<Vec<String>, String> {
    if pointer.is_empty() {
        return Ok(Vec::new());
    }
    let Some(rest) = pointer.strip_prefix('/') else {
        return Err(format!("`{pointer}` is not a JSON pointer"));
    };
    Ok(rest
        .split('/')
        .map(|t| t.replace("~1", "/").replace("~0", "~"))
        .collect())
}

fn index(token: &str, len: usize, allow_end: bool) -> Result<usize, String> {
    if token == "-" && allow_end {
        return Ok(len);
    }
    let valid = !token.is_empty()
        && token.bytes().all(|b| b.is_ascii_digit())
        && (token == "0" || !token.starts_with('0'));
    let i: usize = valid
        .then(|| token.parse().ok())
        .flatten()
        .ok_or_else(|| format!("`{token}` is not an array index"))?;
    let max = if allow_end {
        len
    } else {
        len.saturating_sub(1)
    };
    if i > max || (!allow_end && len == 0) {
        return Err(format!("index {i} is outside the array"));
    }
    Ok(i)
}

fn get<'a>(doc: &'a Value, tokens: &[String]) -> Option<&'a Value> {
    tokens.iter().try_fold(doc, |node, token| match node {
        Value::Object(map) => map.get(token),
        Value::Array(items) => index(token, items.len(), false)
            .ok()
            .and_then(|i| items.get(i)),
        _ => None,
    })
}

fn get_mut<'a>(doc: &'a mut Value, tokens: &[String]) -> Option<&'a mut Value> {
    tokens.iter().try_fold(doc, |node, token| match node {
        Value::Object(map) => map.get_mut(token),
        Value::Array(items) => {
            let i = index(token, items.len(), false).ok()?;
            items.get_mut(i)
        }
        _ => None,
    })
}

fn add(doc: &mut Value, path: &str, value: Value) -> Result<(), String> {
    let tokens = tokens(path)?;
    let Some((last, parents)) = tokens.split_last() else {
        *doc = value;
        return Ok(());
    };
    let parent = get_mut(doc, parents).ok_or_else(|| missing(path))?;
    match parent {
        Value::Object(map) => {
            map.insert(last.clone(), value);
            Ok(())
        }
        Value::Array(items) => {
            let i = index(last, items.len(), true)?;
            items.insert(i, value);
            Ok(())
        }
        _ => Err(format!("the parent of {path} is not an object or array")),
    }
}

fn remove(doc: &mut Value, path: &str) -> Result<Value, String> {
    let tokens = tokens(path)?;
    let Some((last, parents)) = tokens.split_last() else {
        return Err("the whole spec cannot be removed".to_owned());
    };
    let parent = get_mut(doc, parents).ok_or_else(|| missing(path))?;
    match parent {
        Value::Object(map) => map.remove(last).ok_or_else(|| missing(path)),
        Value::Array(items) => {
            let i = index(last, items.len(), false)?;
            Ok(items.remove(i))
        }
        _ => Err(missing(path)),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn op(json: Value) -> PatchOp {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn the_rfc_6902_appendix_examples_apply() {
        let doc = json!({ "foo": "bar", "baz": ["a", "c"], "q": { "bar": 2 } });
        let out = apply(
            &doc,
            &[
                op(json!({ "op": "add", "path": "/baz/1", "value": "b" })),
                op(json!({ "op": "add", "path": "/baz/-", "value": "d" })),
                op(json!({ "op": "replace", "path": "/foo", "value": "qux" })),
                op(json!({ "op": "move", "from": "/q/bar", "path": "/moved" })),
                op(json!({ "op": "copy", "from": "/baz/0", "path": "/first" })),
                op(json!({ "op": "remove", "path": "/q" })),
                op(json!({ "op": "test", "path": "/moved", "value": 2 })),
                op(json!({ "op": "add", "path": "/a~1b", "value": 1 })),
            ],
        )
        .unwrap();
        assert_eq!(
            out,
            json!({ "foo": "qux", "baz": ["a", "b", "c", "d"], "moved": 2, "first": "a", "a/b": 1 })
        );
    }

    #[test]
    fn a_failing_operation_changes_nothing() {
        let doc = json!({ "a": [1, 2] });
        for bad in [
            json!({ "op": "remove", "path": "/missing" }),
            json!({ "op": "replace", "path": "/a/5", "value": 0 }),
            json!({ "op": "add", "path": "/a/01", "value": 0 }),
            json!({ "op": "test", "path": "/a/0", "value": 9 }),
            json!({ "op": "move", "from": "/a", "path": "/a/0" }),
            json!({ "op": "add", "path": "nope", "value": 0 }),
        ] {
            let ops = [
                op(json!({ "op": "add", "path": "/b", "value": 1 })),
                op(bad.clone()),
            ];
            assert!(apply(&doc, &ops).is_err(), "{bad}");
        }
    }
}
