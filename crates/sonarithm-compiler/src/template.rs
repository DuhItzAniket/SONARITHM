// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Template expansion (RFC-0004 §2).
//!
//! After the source is read and the spec's own `transform` has run, the template's provider
//! binds its slots to the data and returns the sections it designs. They merge with the
//! spec as written (before defaults):
//!
//! - a top-level section written in the spec replaces the template's section of the same
//!   name, as a whole;
//! - `transform` is the exception: the spec's transforms run first, then the template's;
//! - in `data`, the template's `fields` fill in only when the spec declares none, and a
//!   computed `source` (values the template derived from the data, such as
//!   `box-five-number`'s quartiles) replaces the data read.
//!
//! The merged spec, without `template`, is validated and given its defaults again, then
//! compiled. The Score keeps the hash of the spec as written: `template` stays symbolic in
//! the canonical form, and the engine versions in the header pin what it produced.

use serde_json::{Map, Value};
use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};
use sonarithm_core::engine::{BindingError, CompileError, TemplateExpansion};
use sonarithm_spec::v02::TemplateRef;
use sonarithm_spec::{Spec, Transform};

use crate::codes;

/// A spec merged with its template's sections.
pub(crate) struct Merged {
    /// The merged spec as JSON, without `template`.
    pub value: Value,
    /// The template's transforms, which run after the spec's.
    pub transforms: Vec<Transform>,
    /// Whether the template computed the data source.
    pub computed_source: bool,
    /// The 033 and 034 `Info` diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

fn at_template() -> SpecPointer {
    SpecPointer::root().key("template")
}

/// The `SNR-COMP-030` diagnostic for an ID no engine registered.
pub(crate) fn unknown(template: &TemplateRef, known: &[&str]) -> Diagnostic {
    let list = if known.is_empty() {
        "none are registered in this build".to_owned()
    } else {
        format!("the registered templates are {}", known.join(", "))
    };
    Diagnostic::new(
        &codes::UNKNOWN_TEMPLATE,
        format!(
            "No engine registers the template `{}`; {list}.",
            template.id
        ),
    )
    .at(at_template().key("id"))
}

/// The diagnostic for a binding error, or `None` for any other error.
pub(crate) fn binding_diagnostic(error: &CompileError) -> Option<Diagnostic> {
    let CompileError::Binding { template, error } = error else {
        return None;
    };
    let bind = at_template().key("bind");
    Some(match error {
        BindingError::Invalid {
            slot,
            field,
            reason,
        } => Diagnostic::new(
            &codes::INVALID_BINDING,
            format!("Template `{template}` cannot bind slot `{slot}` to field `{field}`: {reason}."),
        )
        .at(bind.key(slot)),
        BindingError::Unbound { slot, kind } => Diagnostic::new(
            &codes::UNBOUND_SLOT,
            format!(
                "Template `{template}` needs a {kind} field for slot `{slot}`, and the data has none left."
            ),
        )
        .at(bind),
        _ => Diagnostic::new(
            &codes::INVALID_BINDING,
            format!("Template `{template}`: {error}."),
        )
        .at(bind),
    })
}

fn written(value: &Map<String, Value>, key: &str) -> bool {
    value.get(key).is_some_and(|v| match v {
        Value::Array(items) => !items.is_empty(),
        Value::Object(entries) => !entries.is_empty(),
        Value::Null => false,
        _ => true,
    })
}

/// Merges `expansion` into the spec as written.
#[expect(
    clippy::too_many_lines,
    reason = "one match over the section names reads better than a helper per section"
)]
///
/// # Errors
///
/// A diagnostic when the template's sections are not a JSON object of sections, which is a
/// bug in the template.
pub(crate) fn merge(
    spec: &Spec,
    template: &TemplateRef,
    expansion: &TemplateExpansion,
) -> Result<Merged, Box<Diagnostic>> {
    let bug = |what: &str| {
        Box::new(
            Diagnostic::new(
                &codes::INVARIANT_BROKEN,
                format!(
                    "Template `{}` produced {what}. This is an engine bug; please report it.",
                    template.id
                ),
            )
            .at(at_template()),
        )
    };
    let Ok(Value::Object(mut merged)) = serde_json::to_value(spec) else {
        return Err(bug("a spec that cannot be read back"));
    };
    let Ok(Value::Object(sections)) = serde_json::from_str::<Value>(expansion.sections.json())
    else {
        return Err(bug("sections that are not a JSON object"));
    };
    merged.remove("template");
    let mut taken: Vec<String> = Vec::new();
    let mut kept: Vec<String> = Vec::new();
    let mut transforms = Vec::new();
    let mut computed_source = false;
    for (key, value) in sections {
        match key.as_str() {
            "transform" => {
                transforms = serde_json::from_value::<Vec<Transform>>(value.clone())
                    .map_err(|_| bug("a `transform` that is not a transform list"))?;
                let mut all = match merged.remove("transform") {
                    Some(Value::Array(own)) => own,
                    _ => Vec::new(),
                };
                if let Value::Array(more) = value {
                    if !more.is_empty() {
                        taken.push("transform".to_owned());
                    }
                    all.extend(more);
                }
                if !all.is_empty() {
                    merged.insert("transform".to_owned(), Value::Array(all));
                }
            }
            "data" => {
                let Value::Object(data) = value else {
                    return Err(bug("a `data` section that is not an object"));
                };
                let Some(Value::Object(own)) = merged.get_mut("data") else {
                    return Err(bug("a spec without `data`"));
                };
                for (part, value) in data {
                    let name = format!("data.{part}");
                    match part.as_str() {
                        "source" => {
                            own.insert(part, value);
                            computed_source = true;
                            taken.push(name);
                        }
                        "fields" if written(own, "fields") => kept.push(name),
                        "fields" => {
                            own.insert(part, value);
                            taken.push(name);
                        }
                        _ => return Err(bug(&format!("an unexpected `{name}`"))),
                    }
                }
            }
            "substrate" | "tracks" | "context" | "music" | "playback" | "output" => {
                if written(&merged, &key) {
                    kept.push(key);
                } else {
                    merged.insert(key.clone(), value);
                    taken.push(key);
                }
            }
            _ => return Err(bug(&format!("an unexpected section `{key}`"))),
        }
    }
    let binding = expansion
        .bindings
        .iter()
        .map(|(slot, field)| format!("{slot} = `{field}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut diagnostics = vec![
        Diagnostic::new(
            &codes::TEMPLATE_BINDING,
            format!("Template `{}` binds {binding}.", template.id),
        )
        .at(at_template().key("bind")),
    ];
    let mut parts = Vec::new();
    if !taken.is_empty() {
        parts.push(format!("takes {} from the template", taken.join(", ")));
    }
    if !kept.is_empty() {
        parts.push(format!(
            "keeps its own {} in place of the template's",
            kept.join(", ")
        ));
    }
    if !parts.is_empty() {
        diagnostics.push(
            Diagnostic::new(
                &codes::TEMPLATE_SECTIONS,
                format!("The spec {}.", parts.join(" and ")),
            )
            .at(at_template()),
        );
    }
    Ok(Merged {
        value: Value::Object(merged),
        transforms,
        computed_source,
        diagnostics,
    })
}
