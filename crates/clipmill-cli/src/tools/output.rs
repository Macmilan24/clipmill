//! Stable fields of successful tool replies. New result fields are additive.

use serde_json::{Value, json};

fn record(fields: &[(&str, &str)]) -> Value {
    let properties: serde_json::Map<_, _> = fields
        .iter()
        .map(|(name, kind)| {
            let value = if *kind == "array" {
                json!({"type":"array","items":{"type":"object"}})
            } else {
                json!({"type":kind})
            };
            ((*name).to_owned(), value)
        })
        .collect();
    let required: Vec<_> = fields.iter().map(|(name, _)| *name).collect();
    json!({"type":"object","properties":properties,"required":required})
}

fn list(name: &str, fields: &[(&str, &str)]) -> Value {
    let mut result = record(&[(name, "array")]);
    result["properties"][name]["items"] = record(fields);
    result
}

pub(super) fn schema(operation: &str) -> Value {
    match operation {
        "status" => record(&[
            ("engine", "string"),
            ("version", "string"),
            ("operations", "array"),
        ]),
        "list_projects" => list("projects", &[("project_id", "string"), ("name", "string")]),
        "create_project" => record(&[("project_id", "string"), ("name", "string")]),
        "add_source" | "get_source" => record(&[
            ("source_id", "string"),
            ("project_id", "string"),
            ("bytes", "integer"),
        ]),
        "list_sources" => list(
            "sources",
            &[
                ("source_id", "string"),
                ("path", "string"),
                ("missing", "boolean"),
            ],
        ),
        "get_job" | "cancel_job" | "start_analysis" | "" => record(&[
            ("job_id", "string"),
            ("state", "string"),
            ("progress", "object"),
            ("stages", "array"),
        ]),
        "list_jobs" => list(
            "jobs",
            &[
                ("job_id", "string"),
                ("kind", "string"),
                ("state", "string"),
            ],
        ),
        "get_transcript" => list(
            "sentences",
            &[("text", "string"), ("start", "number"), ("end", "number")],
        ),
        "find_moments" => list(
            "moments",
            &[("text", "string"), ("start", "number"), ("end", "number")],
        ),
        "memory_overview" => record(&[
            ("job_id", "string"),
            ("chapters", "array"),
            ("events", "array"),
            ("links", "array"),
            ("entities", "array"),
        ]),
        "get_timeline" => list(
            "artifacts",
            &[("kind", "string"), ("artifact_id", "string")],
        ),
        "audit_log" => list(
            "entries",
            &[
                ("client", "string"),
                ("op", "string"),
                ("outcome", "string"),
            ],
        ),
        "job_events" => record(&[
            ("job_id", "string"),
            ("events", "array"),
            ("cursor", "integer"),
        ]),
        "list_models" => list("models", &[("name", "string")]),
        _ => edit_schema(operation),
    }
}

fn edit_schema(operation: &str) -> Value {
    match operation {
        "list_clips" => list(
            "clips",
            &[
                ("clip_id", "string"),
                ("kind", "string"),
                ("decision", "string"),
            ],
        ),
        "describe_clip" => record(&[
            ("clip_id", "string"),
            ("kind", "string"),
            ("decision", "string"),
        ]),
        "set_decision" => record(&[("clip_id", "string"), ("decision", "string")]),
        "create_clip" => record(&[
            ("doc_id", "string"),
            ("revision", "integer"),
            ("reopened", "boolean"),
        ]),
        "get_edit" => record(&[
            ("doc_id", "string"),
            ("revision", "integer"),
            ("document", "object"),
        ]),
        "apply_edits" | "undo_edit" => record(&[
            ("doc_id", "string"),
            ("revision", "integer"),
            ("inverse", "object"),
        ]),
        "edit_log" => list(
            "entries",
            &[
                ("revision", "integer"),
                ("command", "object"),
                ("inverse", "object"),
            ],
        ),
        "export_clip" => record(&[("job_id", "string"), ("destination_dir", "string")]),
        _ => json!({"type":"object"}),
    }
}

#[cfg(test)]
mod tests {
    use super::schema;

    #[test]
    fn every_tool_declares_its_success_fields() {
        for tool in super::super::TOOLS {
            let output = schema(tool.op);
            assert!(
                output["required"].as_array().is_some_and(|f| !f.is_empty()),
                "{}",
                tool.name
            );
        }
    }
}
