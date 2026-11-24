//! The MCP tools: each one engine API operation, described for an agent.
//!
//! Descriptions say when to use a tool and what to ask the person first,
//! because an agent reads them to decide. Arguments mirror the engine API
//! (docs/engine-api.md) so a tool call is that operation and nothing more.

mod output;

use serde_json::{Value, json};

#[derive(Debug)]
pub struct Tool {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// The engine operation it calls; empty for the tools the server answers.
    pub op: &'static str,
    pub read_only: bool,
    pub destructive: bool,
    pub input: fn() -> Value,
}

fn object(properties: &Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

fn string(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

fn run_args() -> Value {
    json!({
        "project_id": string("The project, from clipmill_list_projects."),
        "source_id": string("The recording, from clipmill_add_source or clipmill_list_sources."),
        "job_id": string("Optional: a specific analysis run. Without it, the newest finished analysis."),
    })
}

fn with(mut base: Value, extra: &Value) -> Value {
    if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    base
}

fn proposal_args() -> Value {
    let args = with(
        run_args(),
        &json!({
            "prompt": {"type":"string","minLength":1,"maxLength":1000,"description":"What should the clips focus on?"},
            "count": {"type":"integer","minimum":1,"maximum":30},
            "min_seconds": {"type":"number","minimum":0},
            "max_seconds": {"type":"number","exclusiveMinimum":0}
        }),
    );
    object(&args, &["project_id", "source_id", "prompt"])
}

/// A brief, or a round of answers to the questions an earlier run listed.
fn reprompt_args() -> Value {
    let mut schema = proposal_args();
    schema["properties"]["answers"] = json!({
        "type": "array",
        "maxItems": 2000,
        "description": "Answers to questions from clipmill_judge_questions. Earlier answers are kept; without a prompt the earlier brief is kept too.",
        "items": {
            "type": "object",
            "required": ["question_id", "answer"],
            "additionalProperties": false,
            "properties": {
                "question_id": {"type": "string", "pattern": "^q_[0-9a-f]{24}$"},
                "answer": {
                    "description": "yes, no, or the probability that the answer is yes.",
                    "anyOf": [
                        {"type": "string", "enum": ["yes", "no"]},
                        {"type": "number", "minimum": 0, "maximum": 1}
                    ]
                }
            }
        }
    });
    schema["required"] = json!(["project_id", "source_id"]);
    schema["anyOf"] = json!([{"required": ["prompt"]}, {"required": ["answers"]}]);
    schema
}

fn edit_args() -> Value {
    json!({"doc_id":string("The edit document from clipmill_create_clip or clipmill_create_clip_from_plan.")})
}

fn revision_args() -> Value {
    with(
        edit_args(),
        &json!({"expected_revision":{"type":"integer","minimum":0,"description":"The revision returned by clipmill_get_edit."}}),
    )
}

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "clipmill_status",
        title: "ClipMill status",
        description: "Check that ClipMill is running on this computer and see its version and the operations available.",
        op: "status",
        read_only: true,
        destructive: false,
        input: || object(&json!({}), &[]),
    },
    Tool {
        name: "clipmill_list_projects",
        title: "List projects",
        description: "The person's ClipMill projects. Each holds recordings and the clips made from them.",
        op: "list_projects",
        read_only: true,
        destructive: false,
        input: || object(&json!({}), &[]),
    },
    Tool {
        name: "clipmill_create_project",
        title: "Create a project",
        description: "Make a new project to hold a recording and its clips.",
        op: "create_project",
        read_only: false,
        destructive: false,
        input: || {
            object(
                &json!({ "name": string("What to call the project.") }),
                &["name"],
            )
        },
    },
    Tool {
        name: "clipmill_add_source",
        title: "Add a recording",
        description: "Add a video or audio file on this computer to a project. The file stays where it is; ClipMill reads it in place.",
        op: "add_source",
        read_only: false,
        destructive: false,
        input: || {
            object(
                &json!({
                    "project_id": string("The project to add it to."),
                    "path": string("The absolute path of the file on this computer."),
                }),
                &["project_id", "path"],
            )
        },
    },
    Tool {
        name: "clipmill_list_sources",
        title: "List recordings",
        description: "The recordings in a project.",
        op: "list_sources",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &json!({ "project_id": string("The project.") }),
                &["project_id"],
            )
        },
    },
    Tool {
        name: "clipmill_analyze",
        title: "Analyze a recording",
        description: "Start finding clips in a recording: transcription, voices, audio events, the recording's memory and clips built from distant moments. Runs entirely on this computer and takes minutes for a long recording; it returns a job at once. Follow it with clipmill_wait_for_job, then clipmill_list_clips.",
        op: "start_analysis",
        read_only: false,
        destructive: false,
        input: || {
            object(
                &json!({
                    "project_id": string("The project."),
                    "source_id": string("The recording."),
                    "min_seconds": { "type": "number", "description": "Optional: the shortest clip wanted." },
                    "max_seconds": { "type": "number", "description": "Optional: the longest clip wanted." },
                    "language": string("Optional: the spoken language as a BCP 47 code, such as en."),
                    "editorial": {"type":"string","enum":["local","none"],"description":"Use the on-device editorial model or skip it."},
                    "prompt": {"type":"string","maxLength":1000,"description":"An editorial brief for the clips."},
                    "count": {"type":"integer","minimum":1,"maximum":30},
                }),
                &["project_id", "source_id"],
            )
        },
    },
    Tool {
        name: "clipmill_get_job",
        title: "Check a job",
        description: "How far an analysis or export has got: its state and which stages are done.",
        op: "get_job",
        read_only: true,
        destructive: false,
        input: || object(&json!({ "job_id": string("The job.") }), &["job_id"]),
    },
    Tool {
        name: "clipmill_wait_for_job",
        title: "Wait for a job",
        description: "Wait up to `seconds` (at most 120) for a job to finish, and return it as it then stands. Call again while it is still running.",
        op: "",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &json!({
                    "job_id": string("The job."),
                    "seconds": { "type": "number", "description": "How long to wait, up to 120." },
                }),
                &["job_id"],
            )
        },
    },
    Tool {
        name: "clipmill_list_jobs",
        title: "List jobs",
        description: "The analyses and exports of a project.",
        op: "list_jobs",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &json!({ "project_id": string("The project.") }),
                &["project_id"],
            )
        },
    },
    Tool {
        name: "clipmill_cancel_job",
        title: "Cancel a job",
        description: "Stop an analysis or export that is still running.",
        op: "cancel_job",
        read_only: false,
        destructive: false,
        input: || object(&json!({ "job_id": string("The job.") }), &["job_id"]),
    },
    Tool {
        name: "clipmill_get_transcript",
        title: "Read the transcript",
        description: "Read sentences, speakers and unit IDs (s12). Use the source_fingerprint and exact start_ticks and end_ticks (90,000 per second) to build a clip plan. Narrow the results with start_seconds and end_seconds.",
        op: "get_transcript",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &with(
                    run_args(),
                    &json!({
                        "start_seconds": { "type": "number" },
                        "end_seconds": { "type": "number" },
                    }),
                ),
                &["project_id", "source_id"],
            )
        },
    },
    Tool {
        name: "clipmill_find_moments",
        title: "Search a recording",
        description: "Find timecoded passages by transcript, recording memory, and meaning when embeddings are available.",
        op: "find_moments",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &with(
                    run_args(),
                    &json!({
                        "query": {"type":"string","minLength":1,"maxLength":500},
                        "limit": {"type":"integer","minimum":1,"maximum":30},
                        "filters": object(&json!({"speaker":string("Script speaker, e.g. A."),
                            "chapter":string("Chapter ID or title."), "mood":string("Memory event mood."),
                            "modality":string("speech, audio, or an audio event such as laughter.")}), &[])
                    }),
                ),
                &["project_id", "source_id", "query"],
            )
        },
    },
    Tool {
        name: "clipmill_memory_overview",
        title: "Read recording memory",
        description: "Read a recording's chapters, events, named entities and links between moments.",
        op: "memory_overview",
        read_only: true,
        destructive: false,
        input: || object(&run_args(), &["project_id", "source_id"]),
    },
    Tool {
        name: "clipmill_propose_clips",
        title: "Propose clips for a brief",
        description: "Ask for finished clip plans from an analyzed recording. Give a concrete brief such as every time a topic comes up. Returns a job you can follow.",
        op: "propose_clips",
        read_only: false,
        destructive: false,
        input: proposal_args,
    },
    Tool {
        name: "clipmill_reprompt",
        title: "Refine a recording's clips",
        description: "Give an existing analysis a new brief or duration, or answers to the questions it listed. Reuses the recording's observations and preserves the earlier run. Returns a job.",
        op: "reprompt",
        read_only: false,
        destructive: false,
        input: reprompt_args,
    },
    Tool {
        name: "clipmill_judge_questions",
        title: "Questions for your judgement",
        description: "Narrow yes-or-no questions ClipMill's editorial judge asked about an analysis, such as whether a clip ends before its speaker has finished, or which of two versions makes the better short. Answer them with clipmill_reprompt to have your judgement, rather than the local model's, decide those edits. ClipMill first checks your answers on questions the recording itself answers.",
        op: "judge_questions",
        read_only: true,
        destructive: false,
        input: || object(&run_args(), &["project_id", "source_id"]),
    },
    Tool {
        name: "clipmill_create_clip_from_plan",
        title: "Create an edit from a plan",
        description: "Open a ranked plan by clip_id, or supply a complete clipmill.mind.clip_plan.v1 plan drawn from this recording's legal source boundaries. Returns the edit and its revision.",
        op: "create_clip_from_plan",
        read_only: false,
        destructive: false,
        input: || {
            let args = with(
                run_args(),
                &json!({
                    "clip_id": string("A plan id from clipmill_list_clips."),
                    "plan": {"type":"object","description":"A complete clipmill.mind.clip_plan.v1 document."},
                    "shape": {"type":"string","enum":["vertical","square","portrait","landscape"]}
                }),
            );
            let mut schema = object(&args, &["project_id", "source_id"]);
            schema["anyOf"] = json!([{"required":["clip_id"]},{"required":["plan"]}]);
            schema
        },
    },
    Tool {
        name: "clipmill_list_clips",
        title: "List clips",
        description: "The clips an analysis found, best first: plans built by ClipMill's editorial mind (kind \"plan\": a question stitched to its answer, a story with its tangents cut, a strong line moved to the front, a montage) and ranked continuous moments (kind \"moment\"), each with any decision already made.",
        op: "list_clips",
        read_only: true,
        destructive: false,
        input: || object(&run_args(), &["project_id", "source_id"]),
    },
    Tool {
        name: "clipmill_describe_clip",
        title: "Describe a clip",
        description: "One clip with the words of each of its parts, to judge it or explain it to the person.",
        op: "describe_clip",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &with(
                    run_args(),
                    &json!({ "clip_id": string("The clip, from clipmill_list_clips.") }),
                ),
                &["project_id", "source_id", "clip_id"],
            )
        },
    },
    Tool {
        name: "clipmill_set_decision",
        title: "Decide on a clip",
        description: "Record the person's decision on a clip: approved, kept (maybe later), rejected, or undecided to take it back.",
        op: "set_decision",
        read_only: false,
        destructive: false,
        input: || {
            object(
                &with(
                    run_args(),
                    &json!({
                        "clip_id": string("The clip."),
                        "reason": {"type":"string","maxLength":240,"description":"Optional reason to remember for future clips in this project."},
                        "decision": { "type": "string", "enum": ["approved", "kept", "rejected", "undecided"] },
                    }),
                ),
                &["project_id", "source_id", "clip_id", "decision"],
            )
        },
    },
    Tool {
        name: "clipmill_create_clip",
        title: "Make an edit",
        description: "Build an editable clip from a plan or moment: captions, framing and the cuts between parts. Returns a doc_id the person can open in ClipMill's editor or export.",
        op: "create_clip",
        read_only: false,
        destructive: false,
        input: || {
            object(
                &with(
                    run_args(),
                    &json!({
                        "clip_id": string("The clip."),
                        "shape": { "type": "string", "enum": ["vertical", "square", "portrait", "landscape"], "description": "Optional: the frame; vertical by default." },
                    }),
                ),
                &["project_id", "source_id", "clip_id"],
            )
        },
    },
    Tool {
        name: "clipmill_export_clip",
        title: "Export a clip",
        description: "Render an edit to a video file in a folder on this computer. Ask the person first whether they own or may use the recording, and pass what they said as `rights`. Returns a job; wait for it with clipmill_wait_for_job.",
        op: "export_clip",
        read_only: false,
        destructive: false,
        input: || {
            object(
                &json!({
                    "doc_id": string("The edit, from clipmill_create_clip."),
                    "destination_dir": string("The absolute path of the folder to write the file into."),
                    "rights": string("The person's own statement of their right to use this recording."),
                    "title": string("Optional: the file's title."),
                }),
                &["doc_id", "destination_dir", "rights"],
            )
        },
    },
    Tool {
        name: "clipmill_audit_log",
        title: "Read the call log",
        description: "Read this client's recent engine operations and outcomes.",
        op: "audit_log",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &json!({"limit":{"type":"integer","minimum":1,"maximum":2000}}),
                &[],
            )
        },
    },
    Tool {
        name: "clipmill_get_source",
        title: "Get recording details",
        description: "Return the registered recording and its measured streams, duration and source map.",
        op: "get_source",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &json!({"source_id":string("The recording."),"project_id":string("Optional project containing the recording.")}),
                &["source_id"],
            )
        },
    },
    Tool {
        name: "clipmill_get_timeline",
        title: "Inspect timeline layers",
        description: "List the source-aligned transcript, memory and index artifacts in a finished analysis.",
        op: "get_timeline",
        read_only: true,
        destructive: false,
        input: || object(&run_args(), &["project_id", "source_id"]),
    },
    Tool {
        name: "clipmill_job_events",
        title: "Read job progress events",
        description: "Read an ordered page of durable task transitions. Pass the resulting cursor as after for the next page.",
        op: "job_events",
        read_only: true,
        destructive: false,
        input: || {
            object(
                &json!({"job_id":string("The job."),"after":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":500}}),
                &["job_id"],
            )
        },
    },
    Tool {
        name: "clipmill_get_edit",
        title: "Read an edit",
        description: "Read an editable clip, its source analysis and current revision.",
        op: "get_edit",
        read_only: true,
        destructive: false,
        input: || object(&edit_args(), &["doc_id"]),
    },
    Tool {
        name: "clipmill_apply_edits",
        title: "Apply edit commands",
        description: "Apply a batch of ClipMill edit commands together at an observed revision. A stale revision produces a conflict.",
        op: "apply_edits",
        read_only: false,
        destructive: false,
        input: || {
            object(
                &with(
                    revision_args(),
                    &json!({
                        "commands":{"type":"array","minItems":1,"maxItems":100,"items":{"type":"object","required":["op"],"properties":{"op":{"type":"string"}}}}
                    }),
                ),
                &["doc_id", "expected_revision", "commands"],
            )
        },
    },
    Tool {
        name: "clipmill_undo_edit",
        title: "Undo the latest edit",
        description: "Apply the durable inverse of the latest command to the revision you just viewed.",
        op: "undo_edit",
        read_only: false,
        destructive: false,
        input: || object(&revision_args(), &["doc_id", "expected_revision"]),
    },
    Tool {
        name: "clipmill_edit_log",
        title: "Read edit history",
        description: "Read the sequence of editing commands, inverse commands and document revisions.",
        op: "edit_log",
        read_only: true,
        destructive: false,
        input: || object(&edit_args(), &["doc_id"]),
    },
    Tool {
        name: "clipmill_render_preview",
        title: "Render a local working preview",
        description: "Render a frozen edit in the local artifact store for review. Returns a job and the frozen revision.",
        op: "render_preview",
        read_only: false,
        destructive: false,
        input: || object(&revision_args(), &["doc_id", "expected_revision"]),
    },
    Tool {
        name: "clipmill_list_models",
        title: "List models",
        description: "The models ClipMill can use on this computer and which are installed.",
        op: "list_models",
        read_only: true,
        destructive: false,
        input: || object(&json!({}), &[]),
    },
];

/// The `tools/list` entry for a tool.
pub fn listing(tool: &Tool) -> Value {
    json!({
        "name": tool.name,
        "title": tool.title,
        "description": tool.description,
        "inputSchema": (tool.input)(),
        "outputSchema": output::schema(tool.op),
        "annotations": {
            "title": tool.title,
            "readOnlyHint": tool.read_only,
            "destructiveHint": tool.destructive,
            "openWorldHint": false,
        },
    })
}
