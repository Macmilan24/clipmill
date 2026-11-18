//! `clipmill`: ClipMill from a shell, and as an MCP server for agents.
//!
//! ```text
//! clipmill project create "Podcast 42"
//! clipmill add prj_… ~/Movies/episode.mp4
//! clipmill analyze prj_… src_… --wait
//! clipmill clips prj_… src_…
//! clipmill edit prj_… src_… plan_…
//! clipmill export doc_… --to ~/Movies/Clips --rights "My own recording" --wait
//! clipmill mcp            # for Claude Desktop and other MCP clients
//! ```
//!
//! Every command is one or a few engine API calls; `--json` prints the engine's
//! answer as it came.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use clipmill_cli::client::Engine;
use serde_json::{Value, json};

#[derive(Parser)]
#[command(
    name = "clipmill",
    version,
    about = "ClipMill from a shell, and for agents over MCP"
)]
struct Cli {
    /// The engine's socket; by default this account's ClipMill.
    #[arg(long, global = true, env = "CLIPMILL_SOCKET")]
    socket: Option<PathBuf>,
    /// A connected-agent token from Settings. Without one, the local scopes
    /// (read, edit, render) apply.
    #[arg(long, global = true, env = "CLIPMILL_TOKEN", hide_env_values = true)]
    token: Option<String>,
    /// Print the engine's answers as JSON.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Is ClipMill running, and which version.
    Status,
    /// Projects.
    #[command(subcommand)]
    Project(ProjectCommand),
    /// List projects.
    Projects,
    /// Add a recording to a project.
    Add { project: String, path: PathBuf },
    /// The recordings in a project.
    Sources { project: String },
    /// Find clips in a recording, on this computer.
    Analyze {
        project: String,
        source: String,
        /// The shortest clip wanted, in seconds.
        #[arg(long, requires = "max")]
        min: Option<f64>,
        /// The longest clip wanted, in seconds.
        #[arg(long, requires = "min")]
        max: Option<f64>,
        /// Spoken language, such as en.
        #[arg(long)]
        language: Option<String>,
        /// Skip the editorial model and the mind.
        #[arg(long)]
        no_editorial: bool,
        /// Show progress until it finishes.
        #[arg(long)]
        wait: bool,
    },
    /// A job's state; with --wait, follow it to the end.
    Job {
        job: String,
        #[arg(long)]
        wait: bool,
    },
    /// The jobs of a project.
    Jobs { project: String },
    /// Stop a running job.
    Cancel { job: String },
    /// The transcript of an analyzed recording.
    Transcript {
        project: String,
        source: String,
        #[arg(long)]
        run: Option<String>,
    },
    /// Search a recording by its words, memory and meaning.
    Find {
        project: String,
        source: String,
        query: String,
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        speaker: Option<String>,
        #[arg(long)]
        mood: Option<String>,
        #[arg(long)]
        chapter: Option<String>,
        #[arg(long)]
        modality: Option<String>,
        #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u32).range(1..=30))]
        limit: u32,
    },
    /// The clips an analysis found.
    Clips {
        project: String,
        source: String,
        #[arg(long)]
        run: Option<String>,
    },
    /// One clip, with its words.
    Clip {
        project: String,
        source: String,
        clip: String,
        #[arg(long)]
        run: Option<String>,
    },
    /// Approve, keep, reject or undecide a clip.
    Decide {
        project: String,
        source: String,
        clip: String,
        #[arg(value_parser = ["approved", "kept", "rejected", "undecided"])]
        decision: String,
        #[arg(long)]
        run: Option<String>,
    },
    /// Make an edit document from a clip.
    Edit {
        project: String,
        source: String,
        clip: String,
        #[arg(long)]
        run: Option<String>,
        /// vertical, square, portrait or landscape.
        #[arg(long)]
        shape: Option<String>,
    },
    /// Render an edit to a file.
    Export {
        doc: String,
        /// The folder to write into.
        #[arg(long)]
        to: PathBuf,
        /// Your statement of your right to use this recording.
        #[arg(long)]
        rights: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        wait: bool,
    },
    /// Models this computer can use.
    Models,
    /// Any engine API operation, with its arguments as JSON.
    Call {
        op: String,
        #[arg(default_value = "{}")]
        args: String,
    },
    /// Serve ClipMill to an agent over MCP (stdio).
    Mcp,
    /// Serve the engine API and MCP over HTTP on 127.0.0.1, for agents and
    /// scripts that speak HTTP. Every request needs a connected-agent token.
    Serve {
        #[arg(long, default_value_t = clipmill_cli::gateway::DEFAULT_PORT)]
        port: u16,
        /// A browser origin allowed to call, such as `http://localhost:3000`.
        /// None is allowed unless named.
        #[arg(long = "allow-origin")]
        allow_origin: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ProjectCommand {
    /// Make a project.
    Create { name: String },
}

fn run_args(project: &str, source: &str, run: Option<&String>) -> Value {
    let mut args = json!({ "project_id": project, "source_id": source });
    if let Some(run) = run {
        args["job_id"] = json!(run);
    }
    args
}

fn absolute(path: &std::path::Path) -> Result<String, String> {
    std::fs::canonicalize(path)
        .map_err(|error| format!("{}: {error}", path.display()))
        .map(|path| path.display().to_string())
}

fn clock(seconds: f64) -> String {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total = seconds.max(0.0).round() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

/// What a person reads for an answer, when they did not ask for JSON.
fn human(op: &str, value: &Value) -> String {
    match op {
        "status" => format!(
            "ClipMill {} is running{}.",
            value["version"].as_str().unwrap_or("?"),
            if value["local_lock"] == true {
                " with the Local Lock on"
            } else {
                ""
            }
        ),
        "list_projects" => list(&value["projects"], |p| {
            format!(
                "{}  {}",
                p["project_id"].as_str().unwrap_or(""),
                p["name"].as_str().unwrap_or("")
            )
        }),
        "list_sources" => list(&value["sources"], |s| {
            format!(
                "{}  {}",
                s["source_id"].as_str().unwrap_or(""),
                s["path"].as_str().unwrap_or("")
            )
        }),
        "list_jobs" => list(&value["jobs"], |j| {
            format!(
                "{}  {:<12} {}",
                j["job_id"].as_str().unwrap_or(""),
                j["state"].as_str().unwrap_or(""),
                j["kind"].as_str().unwrap_or("")
            )
        }),
        "get_job" | "start_analysis" | "cancel_job" => format!(
            "{}  {}  {}/{} stages",
            value["job_id"].as_str().unwrap_or(""),
            value["state"].as_str().unwrap_or(""),
            value["progress"]["done"],
            value["progress"]["total"]
        ),
        "get_transcript" => list(&value["sentences"], |s| {
            format!(
                "{:>6} {:>6}  {}{}",
                s["unit"].as_str().unwrap_or(""),
                clock(s["start"].as_f64().unwrap_or(0.0)),
                s["speaker"]
                    .as_str()
                    .map(|v| format!("{v}: "))
                    .unwrap_or_default(),
                s["text"].as_str().unwrap_or("")
            )
        }),
        "find_moments" => list(&value["moments"], |m| {
            format!(
                "{}–{}  {}",
                clock(m["start"].as_f64().unwrap_or_default()),
                clock(m["end"].as_f64().unwrap_or_default()),
                m["text"].as_str().unwrap_or_default()
            )
        }),
        "list_clips" => list(&value["clips"], clip_line),
        "describe_clip" => {
            let mut text = clip_line(value);
            if let Some(parts) = value["parts"].as_array() {
                for (number, part) in parts.iter().enumerate() {
                    let _ = write!(
                        text,
                        "\n  {}. {} {}–{}: {}",
                        number + 1,
                        part["role"].as_str().unwrap_or(""),
                        clock(part["start"].as_f64().unwrap_or(0.0)),
                        clock(part["end"].as_f64().unwrap_or(0.0)),
                        part["text"].as_str().unwrap_or("")
                    );
                }
            } else {
                let _ = write!(text, "\n  {}", value["text"].as_str().unwrap_or(""));
            }
            text
        }
        "create_clip" => format!("Edit {} is ready.", value["doc_id"].as_str().unwrap_or("")),
        "export_clip" => format!(
            "Export {} started, writing to {}.",
            value["job_id"].as_str().unwrap_or(""),
            value["destination_dir"].as_str().unwrap_or("")
        ),
        "list_models" => list(&value["models"], |m| {
            format!(
                "{:<34} {:<16} {}",
                m["name"].as_str().unwrap_or(""),
                m["capability"].as_str().unwrap_or(""),
                if m["installed"] == true {
                    "installed"
                } else {
                    ""
                }
            )
        }),
        _ => serde_json::to_string_pretty(value).unwrap_or_default(),
    }
}

fn list(items: &Value, line: impl Fn(&Value) -> String) -> String {
    let lines: Vec<String> = items.as_array().into_iter().flatten().map(line).collect();
    if lines.is_empty() {
        "Nothing yet.".to_owned()
    } else {
        lines.join("\n")
    }
}

fn clip_line(clip: &Value) -> String {
    let title = clip["title"].as_str().unwrap_or("Untitled");
    let decision = clip["decision"]
        .as_str()
        .filter(|d| *d != "undecided")
        .map(|d| format!(" [{d}]"))
        .unwrap_or_default();
    if clip["kind"] == "plan" {
        let parts = clip["parts"].as_array().map_or(0, Vec::len);
        format!(
            "{}  {:<13} {:>5}  {} part{}  {title}{decision}",
            clip["clip_id"].as_str().unwrap_or(""),
            clip["form"].as_str().unwrap_or(""),
            clock(clip["duration"].as_f64().unwrap_or(0.0)),
            parts,
            if parts == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "{}  {:<13} {}–{}  {title}{decision}",
            clip["clip_id"].as_str().unwrap_or(""),
            "moment",
            clock(clip["start"].as_f64().unwrap_or(0.0)),
            clock(clip["end"].as_f64().unwrap_or(0.0)),
        )
    }
}

async fn follow(engine: &Engine, job_id: &str, json: bool) -> Result<Value, String> {
    let mut last = String::new();
    loop {
        let job = call(engine, "get_job", json!({ "job_id": job_id })).await?;
        let state = job["state"].as_str().unwrap_or_default().to_owned();
        let line = format!(
            "{state} {}/{}",
            job["progress"]["done"], job["progress"]["total"]
        );
        if !json && line != last {
            eprintln!("  {line}");
            last = line;
        }
        if matches!(state.as_str(), "succeeded" | "failed" | "cancelled") {
            return Ok(job);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn call(engine: &Engine, op: &str, args: Value) -> Result<Value, String> {
    match engine.call(op, args).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(refusal)) => Err(refusal["message"]
            .as_str()
            .unwrap_or("ClipMill refused the call")
            .to_owned()),
        Err(link) => Err(link.to_string()),
    }
}

/// The engine operation a command is, its arguments, and whether to follow
/// the job it starts.
#[allow(
    clippy::too_many_lines,
    reason = "one arm per command, each a line or two of arguments"
)]
fn request(command: &Command) -> Result<(&str, Value, bool), String> {
    Ok(match command {
        Command::Status => ("status", json!({}), false),
        Command::Projects => ("list_projects", json!({}), false),
        Command::Project(ProjectCommand::Create { name }) => {
            ("create_project", json!({ "name": name }), false)
        }
        Command::Add { project, path } => (
            "add_source",
            json!({ "project_id": project, "path": absolute(path)? }),
            false,
        ),
        Command::Sources { project } => ("list_sources", json!({ "project_id": project }), false),
        Command::Analyze {
            project,
            source,
            min,
            max,
            language,
            no_editorial,
            wait,
        } => {
            let mut args = json!({ "project_id": project, "source_id": source });
            if let (Some(min), Some(max)) = (min, max) {
                args["min_seconds"] = json!(min);
                args["max_seconds"] = json!(max);
            }
            if let Some(language) = language {
                args["language"] = json!(language);
            }
            if *no_editorial {
                args["editorial"] = json!("none");
            }
            ("start_analysis", args, *wait)
        }
        Command::Job { job, wait } => ("get_job", json!({ "job_id": job }), *wait),
        Command::Jobs { project } => ("list_jobs", json!({ "project_id": project }), false),
        Command::Cancel { job } => ("cancel_job", json!({ "job_id": job }), false),
        Command::Transcript {
            project,
            source,
            run,
        } => (
            "get_transcript",
            run_args(project, source, run.as_ref()),
            false,
        ),
        Command::Find {
            project,
            source,
            query,
            run,
            speaker,
            mood,
            chapter,
            modality,
            limit,
        } => {
            let mut args = run_args(project, source, run.as_ref());
            args["query"] = json!(query);
            args["limit"] = json!(limit);
            args["filters"] =
                json!({"speaker": speaker,"mood":mood,"chapter":chapter,"modality":modality});
            ("find_moments", args, false)
        }
        Command::Clips {
            project,
            source,
            run,
        } => ("list_clips", run_args(project, source, run.as_ref()), false),
        Command::Clip {
            project,
            source,
            clip,
            run,
        } => {
            let mut args = run_args(project, source, run.as_ref());
            args["clip_id"] = json!(clip);
            ("describe_clip", args, false)
        }
        Command::Decide {
            project,
            source,
            clip,
            decision,
            run,
        } => {
            let mut args = run_args(project, source, run.as_ref());
            args["clip_id"] = json!(clip);
            args["decision"] = json!(decision);
            ("set_decision", args, false)
        }
        Command::Edit {
            project,
            source,
            clip,
            run,
            shape,
        } => {
            let mut args = run_args(project, source, run.as_ref());
            args["clip_id"] = json!(clip);
            if let Some(shape) = shape {
                args["shape"] = json!(shape);
            }
            ("create_clip", args, false)
        }
        Command::Export {
            doc,
            to,
            rights,
            title,
            wait,
        } => {
            std::fs::create_dir_all(to).map_err(|error| format!("{}: {error}", to.display()))?;
            let mut args =
                json!({ "doc_id": doc, "destination_dir": absolute(to)?, "rights": rights });
            if let Some(title) = title {
                args["title"] = json!(title);
            }
            ("export_clip", args, *wait)
        }
        Command::Models => ("list_models", json!({}), false),
        Command::Call { op, args } => {
            let args: Value = serde_json::from_str(args)
                .map_err(|error| format!("the arguments are not JSON: {error}"))?;
            (op.as_str(), args, false)
        }
        Command::Mcp | Command::Serve { .. } => unreachable!("served above"),
    })
}

async fn run(cli: Cli) -> Result<(), String> {
    let engine = Engine::locate(cli.socket, cli.token.unwrap_or_default())?;
    match &cli.command {
        Command::Mcp => {
            return clipmill_cli::mcp::serve(engine)
                .await
                .map_err(|error| error.to_string());
        }
        Command::Serve { port, allow_origin } => {
            return clipmill_cli::gateway::serve(clipmill_cli::gateway::Settings {
                socket: engine.socket().to_path_buf(),
                port: *port,
                allowed_origins: allow_origin.clone(),
            })
            .await
            .map_err(|error| error.to_string());
        }
        _ => {}
    }
    let (op, args, wait) = request(&cli.command)?;
    let mut value = call(&engine, op, args).await?;
    if wait && let Some(job_id) = value["job_id"].as_str().map(str::to_owned) {
        value = follow(&engine, &job_id, cli.json).await?;
    }
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else {
        let shown_op = if wait { "get_job" } else { op };
        println!("{}", human(shown_op, &value));
    }
    if wait && value["state"] != "succeeded" {
        return Err(value["failure"]
            .as_str()
            .unwrap_or("the job did not succeed")
            .to_owned());
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("clipmill: {message}");
            ExitCode::FAILURE
        }
    }
}
