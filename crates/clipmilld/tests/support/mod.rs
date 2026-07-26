#![allow(dead_code, clippy::expect_used)]

use std::{
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{
    CancelJobRequest, CreateEditDocRequest, CreateProjectRequest, DemoDagPayloadV1, GetJobRequest,
    GetSourceRequest, IngestSourcePayloadV1, Job, ListJobsRequest, ListProjectsRequest,
    ListSourcesRequest, PingRequest, ProbeSourcePayloadV1, Project, RegisterSourceRequest,
    RegisterSourceResponse, RenderClipPayloadV1, Request, Response, SnapshotEditDocRequest, Source,
    SubmitJobRequest, request, response,
};
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    time::sleep,
};

pub fn workspace_tempdir() -> tempfile::TempDir {
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    std::fs::create_dir_all(&target).expect("target directory");
    let target = target.canonicalize().expect("canonical target directory");
    tempfile::TempDir::new_in(target).expect("workspace tempdir")
}

pub async fn send(socket: &Path, request: Request) -> Result<Response, String> {
    let mut stream = UnixStream::connect(socket)
        .await
        .map_err(|error| error.to_string())?;
    send_on_stream(&mut stream, request).await?;
    read_response(&mut stream).await
}

pub async fn send_on_stream(stream: &mut UnixStream, request: Request) -> Result<(), String> {
    let mut encoded = Vec::new();
    request
        .encode_length_delimited(&mut encoded)
        .map_err(|error| error.to_string())?;
    stream
        .write_all(&encoded)
        .await
        .map_err(|error| error.to_string())
}

pub async fn read_response(stream: &mut UnixStream) -> Result<Response, String> {
    let length = read_varint(stream).await?;
    let length = usize::try_from(length).map_err(|error| error.to_string())?;
    let mut response = vec![0_u8; length];
    stream
        .read_exact(&mut response)
        .await
        .map_err(|error| error.to_string())?;
    Response::decode(response.as_slice()).map_err(|error| error.to_string())
}

pub async fn send_without_reading_response(socket: &Path, request: Request) -> Result<(), String> {
    let mut stream = UnixStream::connect(socket)
        .await
        .map_err(|error| error.to_string())?;
    let mut encoded = Vec::new();
    request
        .encode_length_delimited(&mut encoded)
        .map_err(|error| error.to_string())?;
    stream
        .write_all(&encoded)
        .await
        .map_err(|error| error.to_string())?;
    stream.shutdown().await.map_err(|error| error.to_string())
}

pub async fn wait_until_ready(socket: &Path) -> Result<(), String> {
    for attempt in 0..350 {
        let request = Request {
            request_id: format!("ready-{attempt}"),
            body: Some(request::Body::Ping(PingRequest {
                echo: "ready".to_owned(),
            })),
        };
        if send(socket, request).await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(20)).await;
    }
    Err("daemon did not become ready".to_owned())
}

pub async fn create(socket: &Path, request_id: &str, name: &str) -> Result<Project, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::CreateProject(CreateProjectRequest {
                name: name.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::CreateProject(created)) => created
            .project
            .ok_or_else(|| "create response omitted project".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected create response".to_owned()),
    }
}

pub async fn list(socket: &Path, request_id: &str) -> Result<Vec<Project>, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::ListProjects(ListProjectsRequest {})),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::ListProjects(listed)) => Ok(listed.projects),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected list response".to_owned()),
    }
}

pub async fn submit_demo(
    socket: &Path,
    request_id: &str,
    project_id: &str,
    payload: &[u8],
) -> Result<Job, String> {
    let payload = DemoDagPayloadV1 {
        key_version: "clipmill.demo-dag.v1".to_owned(),
        seed: payload.to_vec(),
    }
    .encode_to_vec();
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::SubmitJob(SubmitJobRequest {
                project_id: project_id.to_owned(),
                kind: "demo-dag".to_owned(),
                payload,
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::SubmitJob(submitted)) => submitted
            .job
            .ok_or_else(|| "submit response omitted job".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected submit response".to_owned()),
    }
}

pub async fn register_source(
    socket: &Path,
    request_id: &str,
    project_id: &str,
    absolute_path: &Path,
) -> Result<RegisterSourceResponse, String> {
    let path = absolute_path
        .to_str()
        .ok_or_else(|| "test source path is not UTF-8".to_owned())?;
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::RegisterSource(RegisterSourceRequest {
                project_id: project_id.to_owned(),
                absolute_path: path.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::RegisterSource(registered)) => Ok(registered),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected register source response".to_owned()),
    }
}

pub async fn get_source(
    socket: &Path,
    request_id: &str,
    source_id: &str,
) -> Result<Source, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::GetSource(GetSourceRequest {
                source_id: source_id.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::GetSource(fetched)) => fetched
            .source
            .ok_or_else(|| "get source response omitted source".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected get source response".to_owned()),
    }
}

pub async fn list_sources(
    socket: &Path,
    request_id: &str,
    project_id: &str,
) -> Result<Vec<Source>, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::ListSources(ListSourcesRequest {
                project_id: project_id.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::ListSources(listed)) => Ok(listed.sources),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected list sources response".to_owned()),
    }
}

pub async fn submit_probe(
    socket: &Path,
    request_id: &str,
    project_id: &str,
    source_id: &str,
) -> Result<Job, String> {
    let payload = ProbeSourcePayloadV1 {
        key_version: "clipmill.probe-source.v1".to_owned(),
        source_id: source_id.to_owned(),
    }
    .encode_to_vec();
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::SubmitJob(SubmitJobRequest {
                project_id: project_id.to_owned(),
                kind: "probe-source".to_owned(),
                payload,
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::SubmitJob(submitted)) => submitted
            .job
            .ok_or_else(|| "submit probe response omitted job".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected submit probe response".to_owned()),
    }
}

pub async fn submit_ingest(
    socket: &Path,
    request_id: &str,
    project_id: &str,
    source_id: &str,
) -> Result<Job, String> {
    let payload = IngestSourcePayloadV1 {
        key_version: "clipmill.ingest-source.v1".to_owned(),
        source_id: source_id.to_owned(),
    }
    .encode_to_vec();
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::SubmitJob(SubmitJobRequest {
                project_id: project_id.to_owned(),
                kind: "ingest-source".to_owned(),
                payload,
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::SubmitJob(submitted)) => submitted
            .job
            .ok_or_else(|| "submit ingest response omitted job".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected submit ingest response".to_owned()),
    }
}

pub async fn get_job(socket: &Path, request_id: &str, job_id: &str) -> Result<Job, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::GetJob(GetJobRequest {
                job_id: job_id.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::GetJob(response)) => response
            .job
            .ok_or_else(|| "get job response omitted job".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected get job response".to_owned()),
    }
}

pub async fn cancel_job(socket: &Path, request_id: &str, job_id: &str) -> Result<Job, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::CancelJob(CancelJobRequest {
                job_id: job_id.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::CancelJob(cancelled)) => cancelled
            .job
            .ok_or_else(|| "cancel response omitted job".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected cancel response".to_owned()),
    }
}

pub async fn list_jobs(
    socket: &Path,
    request_id: &str,
    project_id: &str,
) -> Result<Vec<Job>, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::ListJobs(ListJobsRequest {
                project_id: project_id.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::ListJobs(response)) => Ok(response.jobs),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected list jobs response".to_owned()),
    }
}

pub fn spawn_daemon(data_dir: &Path, socket: &Path) -> Child {
    spawn_daemon_with_step_delay(data_dir, socket, None)
}

pub fn spawn_daemon_with_step_delay(
    data_dir: &Path,
    socket: &Path,
    step_delay_ms: Option<u64>,
) -> Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_clipmilld"));
    command
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--socket")
        .arg(socket)
        .env("RUST_LOG", "error")
        .env("CLIPMILL_TEST_BUILTIN_WORKER", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    if let Some(delay) = step_delay_ms {
        command.env("CLIPMILL_W4_STEP_DELAY_MS", delay.to_string());
    }
    command.spawn().expect("spawn clipmilld")
}

pub fn signal_terminate(child: &Child) -> Result<(), String> {
    let status = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("kill -TERM exited with {status}"))
    }
}

pub async fn wait_for_exit(child: &mut Child) -> Result<ExitStatus, String> {
    for _attempt in 0..350 {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Ok(status);
        }
        sleep(Duration::from_millis(20)).await;
    }
    let _kill_result = child.kill();
    let _wait_result = child.wait();
    Err("daemon did not exit within seven seconds".to_owned())
}

async fn read_varint(stream: &mut UnixStream) -> Result<u64, String> {
    let mut value = 0_u64;
    for index in 0..10 {
        let byte = stream.read_u8().await.map_err(|error| error.to_string())?;
        if index == 9 && byte > 1 {
            return Err("malformed response length".to_owned());
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("malformed response length".to_owned())
}

pub async fn create_edit_doc(
    socket: &Path,
    request_id: &str,
    project_id: &str,
    document_json: &str,
) -> Result<String, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::CreateEditDoc(CreateEditDocRequest {
                project_id: project_id.to_owned(),
                document_json: document_json.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::CreateEditDoc(created)) => created
            .doc
            .map(|doc| doc.doc_id)
            .ok_or_else(|| "create edit doc response omitted the document".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected create edit doc response".to_owned()),
    }
}

/// Freeze a document into the immutable snapshot a render reads.
pub async fn snapshot_edit_doc(
    socket: &Path,
    request_id: &str,
    doc_id: &str,
) -> Result<String, String> {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::SnapshotEditDoc(SnapshotEditDocRequest {
                doc_id: doc_id.to_owned(),
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::SnapshotEditDoc(taken)) => Ok(taken.artifact_id),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected snapshot response".to_owned()),
    }
}

pub struct RenderRequest<'a> {
    pub project_id: &'a str,
    pub doc_id: &'a str,
    pub ir_artifact_id: &'a str,
    pub source_attestation: &'a str,
    pub ai_assistance: Vec<String>,
}

pub async fn submit_render(
    socket: &Path,
    request_id: &str,
    render: &RenderRequest<'_>,
) -> Result<Job, String> {
    let payload = RenderClipPayloadV1 {
        key_version: "clipmill.render-clip.v1".to_owned(),
        doc_id: render.doc_id.to_owned(),
        ir_artifact_id: render.ir_artifact_id.to_owned(),
        source_attestation: render.source_attestation.to_owned(),
        gates_passed: vec!["duration_60s".to_owned()],
        ai_assistance: render.ai_assistance.clone(),
    }
    .encode_to_vec();
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::SubmitJob(SubmitJobRequest {
                project_id: render.project_id.to_owned(),
                kind: "render-clip".to_owned(),
                payload,
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::SubmitJob(submitted)) => submitted
            .job
            .ok_or_else(|| "submit render response omitted job".to_owned()),
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected submit render response".to_owned()),
    }
}
