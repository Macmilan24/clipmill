//! Memory search (plan step 2.9): "where do they talk about pricing".
//!
//! Three readings of one analysis, merged: the Recording Memory (events,
//! chapters and named things whose words match), the transcript's full-text
//! index, and, when the recording has embeddings and the meaning search
//! worker is running, closeness in meaning to the query. A moment found more
//! than one way ranks above one found once. Every hit is a span of sentences
//! with its times and words, ready to describe or cut from.

use std::collections::BTreeMap;
use std::time::Duration;

use clipmill_artifacts::ArtifactLease;

use clipmill_contracts::proto::ipc::v1::{
    JobState, SubmitJobRequest, TextEmbedPayloadV1, request, response,
};
use prost::Message;
use serde_json::{Value, json};

use super::Service;
use super::api_v2::{ApiError, ApiResult, Run, arg_str, seconds};

/// How long a search waits for the query's vector before answering from
/// words alone.
const VECTOR_WAIT: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(100);
const DEFAULT_LIMIT: usize = 8;
const MAX_LIMIT: usize = 30;
/// Below this cosine a passage is not about the query, however it ranks.
const MEANING_FLOOR: f32 = 0.78;

/// One place a search found.
#[derive(Clone, Debug)]
struct Hit {
    first: u64,
    last: u64,
    score: f32,
    found_by: Vec<&'static str>,
    label: Option<String>,
}

impl Hit {
    fn relevance(&self) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        let additional = self.found_by.len().saturating_sub(1) as f32;
        self.score + additional * 0.15
    }
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .filter(|w| {
            !matches!(
                w.as_str(),
                "a" | "an"
                    | "the"
                    | "where"
                    | "when"
                    | "what"
                    | "did"
                    | "do"
                    | "does"
                    | "they"
                    | "he"
                    | "she"
                    | "talk"
                    | "about"
                    | "of"
                    | "to"
                    | "is"
                    | "are"
                    | "me"
                    | "show"
                    | "find"
            )
        })
        .collect()
}

/// The share of the query's words that appear in `text`.
fn overlap(query: &[String], text: &str) -> f32 {
    if query.is_empty() {
        return 0.0;
    }
    let text = words(text);
    let found = query.iter().filter(|word| text.contains(word)).count();
    #[allow(clippy::cast_precision_loss)]
    let share = found as f32 / query.len() as f32;
    share
}

fn unit(value: &Value) -> Option<u64> {
    value.as_str()?.strip_prefix('s')?.parse().ok()
}

fn add(hits: &mut BTreeMap<(u64, u64), Hit>, hit: Hit) {
    hits.entry((hit.first, hit.last))
        .and_modify(|known| {
            known.score = known.score.max(hit.score);
            for way in &hit.found_by {
                if !known.found_by.contains(way) {
                    known.found_by.push(way);
                }
            }
            if known.label.is_none() {
                known.label.clone_from(&hit.label);
            }
        })
        .or_insert(hit);
}

/// Filters refer to the evidence on the same analysis as the hit.
#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Filters {
    speaker: Option<String>,
    mood: Option<String>,
    chapter: Option<String>,
    modality: Option<String>,
}

impl Filters {
    fn accepts(
        &self,
        hit: &Hit,
        memory: &Value,
        speakers: &BTreeMap<u64, &str>,
        audio: &[&Value],
        start: u64,
        end: u64,
    ) -> bool {
        let overlaps = |row: &&Value| {
            unit(&row["first_unit"]).is_some_and(|n| n <= hit.last)
                && unit(&row["last_unit"]).is_some_and(|n| n >= hit.first)
        };
        let chapters = memory["chapters"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(overlaps);
        let events = memory["events"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(overlaps);
        self.chapter.as_ref().is_none_or(|name| {
            chapters.clone().any(|row| {
                row["chapter_id"] == *name
                    || row["title"]
                        .as_str()
                        .is_some_and(|s| s.eq_ignore_ascii_case(name))
            })
        }) && self
            .mood
            .as_ref()
            .is_none_or(|mood| events.clone().any(|row| row["mood"] == *mood))
            && self.speaker.as_ref().is_none_or(|speaker| {
                speakers
                    .range(hit.first..=hit.last)
                    .any(|(_, s)| s.eq_ignore_ascii_case(speaker))
            })
            && self.modality.as_deref().is_none_or(|m| match m {
                "speech" => true,
                kind => audio.iter().any(|row| {
                    (kind == "audio" || row["kind"] == kind)
                        && row["start_ticks"].as_u64().is_some_and(|n| n < end)
                        && row["end_ticks"].as_u64().is_some_and(|n| n > start)
                }),
            })
    }
}

fn search_options(args: &Value) -> Result<(Filters, usize), ApiError> {
    let limit = match args.get("limit") {
        None => DEFAULT_LIMIT,
        Some(value) => value
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| (1..=MAX_LIMIT).contains(n))
            .ok_or_else(|| ApiError::invalid("limit must be between 1 and 30"))?,
    };
    let filters: Filters = args
        .get("filters")
        .map(|value| serde_json::from_value(value.clone()))
        .transpose()
        .map_err(|e| ApiError::invalid(format!("invalid search filter: {e}")))?
        .unwrap_or_default();
    Ok((filters, limit))
}

impl Service {
    pub(super) async fn find_moments(&self, args: &Value) -> ApiResult {
        let run = self.run(args).await?;
        let query = arg_str(args, "query")?.trim().to_owned();
        if query.chars().count() > 500 {
            return Err(ApiError::invalid("a search is at most 500 characters"));
        }
        let (filters, limit) = search_options(args)?;
        let terms = words(&query);
        let mut hits = BTreeMap::new();
        let memory = self
            .optional_document(&run, "mind.memory.v1")
            .await?
            .unwrap_or(Value::Null);
        remembered(&memory, &terms, &mut hits);
        let phrase = if query.contains('"') {
            query.clone()
        } else {
            terms.join(" ")
        };
        self.spoken(&run, &phrase, &mut hits).await;
        let meaning = self.meant(&run, &query, &mut hits).await;
        let script = if filters.speaker.is_some() {
            self.optional_document(&run, "mind.script.v1").await?
        } else {
            None
        };
        let speakers = script
            .as_ref()
            .and_then(|s| s["units"].as_array())
            .into_iter()
            .flatten()
            .filter_map(|u| Some((u["index"].as_u64()?, u["speaker"].as_str()?)))
            .collect();
        let audio_doc = if filters.modality.is_some() {
            self.optional_document(&run, "audio.events.v1").await?
        } else {
            None
        };
        let audio = audio_doc
            .as_ref()
            .and_then(|s| s["events"].as_array())
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let index = self.optional_document(&run, "index.transcript.v1").await?;
        let sentences: BTreeMap<u64, &Value> = index
            .as_ref()
            .and_then(|i| i["sentences"].as_array())
            .into_iter()
            .flatten()
            .filter_map(|s| Some((s["index"].as_u64()?, s)))
            .collect();
        let mut ranked: Vec<Hit> = hits.into_values().collect();
        ranked.sort_by(|a, b| {
            b.relevance()
                .total_cmp(&a.relevance())
                .then(a.first.cmp(&b.first))
        });
        let moments: Vec<Value> = ranked
            .into_iter()
            .filter_map(|hit| {
                if hit.last < hit.first {
                    return None;
                }
                let start = sentences.get(&hit.first)?["start_ticks"].as_u64()?;
                let end = sentences.get(&hit.last)?["end_ticks"].as_u64()?;
                if !filters.accepts(&hit, &memory, &speakers, &audio, start, end) {
                    return None;
                }
                let text = sentences
                    .range(hit.first..=hit.last)
                    .filter_map(|(_, sentence)| sentence["text"].as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut moment = json!({
                    "units": format!("s{}-s{}", hit.first, hit.last),
                    "start": seconds(start),
                    "end": seconds(end),
                    "score": (f64::from(hit.relevance()) * 1000.0).round() / 1000.0,
                    "found_by": hit.found_by,
                    "text": text,
                });
                if let Some(label) = hit.label {
                    moment["label"] = json!(label);
                }
                Some(moment)
            })
            .take(limit)
            .collect();
        Ok(json!({
            "job_id": run.job.job_id,
            "query": query,
            "meaning": meaning,
            "moments": moments,
        }))
    }

    /// Full-text matches in the transcript.
    async fn spoken(&self, run: &Run, query: &str, hits: &mut BTreeMap<(u64, u64), Hit>) {
        let Some((lease, path)) = self
            .run_file(run, "timeline.index.v1", "timeline.sqlite")
            .await
        else {
            return;
        };
        let query = query.to_owned();
        let found = tokio::task::spawn_blocking(move || {
            let _lease = lease;
            clipmill_timeline::TimelineIndex::open(&path)
                .and_then(|index| index.search(&query, 256))
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();
        for (rank, row) in found.into_iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let score = 0.8 - rank as f32 * 0.02;
            add(
                hits,
                Hit {
                    first: row.index,
                    last: row.index,
                    score,
                    found_by: vec!["words"],
                    label: None,
                },
            );
        }
    }

    /// Closeness in meaning, when the recording has embeddings and the query
    /// can be embedded now. Says which, so an answer from words alone is not
    /// mistaken for one that also searched by meaning.
    async fn meant(
        &self,
        run: &Run,
        query: &str,
        hits: &mut BTreeMap<(u64, u64), Hit>,
    ) -> &'static str {
        let Some(embeddings_id) = run.artifact("text.embeddings.v1").map(str::to_owned) else {
            return "not_analyzed";
        };
        let Ok(descriptor) = self.document(&run.project_id, &embeddings_id).await else {
            return "unreadable";
        };
        let Some((_lease, vectors)) = self
            .run_file(run, "text.embeddings.v1", "vectors.f16")
            .await
        else {
            return "unreadable";
        };
        if !self.embedding_available("text-embed-query") {
            return "worker_unavailable";
        }
        let Some((digest, query_vector)) = self.query_vector(&run.project_id, query).await else {
            return "worker_unavailable";
        };
        let Ok(bytes) = tokio::fs::read(&vectors).await else {
            return "unreadable";
        };
        let dims = usize::try_from(descriptor["model"]["dims"].as_u64().unwrap_or(0)).unwrap_or(0);
        if descriptor["producer"]["model_digest"] != digest
            || !(1..=4096).contains(&dims)
            || query_vector.len() != dims
            || query_vector.iter().any(|n| !n.is_finite())
        {
            return "mismatched";
        }
        let units = descriptor["units"].as_array().cloned().unwrap_or_default();
        if units.len().checked_mul(dims * 2) != Some(bytes.len()) {
            return "unreadable";
        }
        let mut scored: Vec<(f32, &Value)> = units
            .iter()
            .enumerate()
            .filter_map(|(row, unit)| {
                let offset = row * dims * 2;
                let slice = bytes.get(offset..offset + dims * 2)?;
                let dot: f32 = slice
                    .chunks_exact(2)
                    .zip(&query_vector)
                    .map(|(half, q)| f16_to_f32(u16::from_le_bytes([half[0], half[1]])) * q)
                    .sum();
                dot.is_finite().then_some((dot, unit))
            })
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (score, unit) in scored.into_iter().filter(|(s, _)| *s >= MEANING_FLOOR) {
            let (Some(first), Some(last)) =
                (unit["first_unit"].as_u64(), unit["last_unit"].as_u64())
            else {
                continue;
            };
            add(
                hits,
                Hit {
                    first,
                    last,
                    score,
                    found_by: vec!["meaning"],
                    label: (unit["kind"] == "event")
                        .then(|| unit["id"].as_str().unwrap_or("").to_owned()),
                },
            );
        }
        "searched"
    }

    /// The query as a vector, from the meaning search worker, or nothing if
    /// it cannot be had in time.
    async fn query_vector(&self, project_id: &str, query: &str) -> Option<(String, Vec<f32>)> {
        let payload = TextEmbedPayloadV1 {
            key_version: crate::jobs::TEXT_EMBED_KEY_VERSION.to_owned(),
            stage: "text-embed-query".to_owned(),
            source_fingerprint: String::new(),
            query: query.to_owned(),
        };
        let body = self
            .v1(
                request::Body::SubmitJob(SubmitJobRequest {
                    project_id: project_id.to_owned(),
                    kind: "embed-query".to_owned(),
                    payload: payload.encode_to_vec(),
                }),
                None,
            )
            .await
            .ok()?;
        let response::Body::SubmitJob(submitted) = body else {
            return None;
        };
        let job_id = submitted.job?.job_id;
        let deadline = tokio::time::Instant::now() + VECTOR_WAIT;
        loop {
            let job = self.job(&job_id).await.ok()?;
            if job.state == JobState::Succeeded as i32 {
                let artifact = job.tasks.first()?.output_artifact_id.clone();
                let (_lease, path) = self.artifact_file(&artifact, "vector.json").await?;
                let document: Value =
                    serde_json::from_slice(&tokio::fs::read(path).await.ok()?).ok()?;
                #[allow(clippy::cast_possible_truncation)]
                let vector = document["vector"]
                    .as_array()?
                    .iter()
                    .map(|value| value.as_f64().map(|n| n as f32))
                    .collect::<Option<Vec<_>>>()?;
                return Some((document["model_digest"].as_str()?.to_owned(), vector));
            }
            if job.state == JobState::Failed as i32
                || job.state == JobState::Cancelled as i32
                || tokio::time::Instant::now() >= deadline
            {
                // A search that could not be embedded in time is answered from
                // its words; the queued work is not left behind.
                let _ = self
                    .v1(
                        request::Body::CancelJob(
                            clipmill_contracts::proto::ipc::v1::CancelJobRequest { job_id },
                        ),
                        None,
                    )
                    .await;
                return None;
            }
            tokio::time::sleep(POLL).await;
        }
    }

    async fn run_file(
        &self,
        run: &Run,
        kind: &str,
        file: &str,
    ) -> Option<(ArtifactLease, std::path::PathBuf)> {
        let artifact = run.artifact(kind)?.to_owned();
        self.artifact_file(&artifact, file).await
    }

    /// One file of a published artifact, verified, on disk.
    async fn artifact_file(
        &self,
        artifact_id: &str,
        file: &str,
    ) -> Option<(ArtifactLease, std::path::PathBuf)> {
        let artifacts = self.artifacts.as_ref()?;
        let lease = artifacts.open(artifact_id.parse().ok()?).await.ok()?;
        let path = crate::media::artifact_path(file).ok()?;
        let file = lease.verified_path(&path).ok()?;
        Some((lease, file))
    }
}

/// Events, chapters and named things whose words match.
fn remembered(memory: &Value, terms: &[String], hits: &mut BTreeMap<(u64, u64), Hit>) {
    for event in memory["events"].as_array().into_iter().flatten() {
        let share = overlap(terms, event["title"].as_str().unwrap_or(""));
        if share >= 0.5
            && let (Some(first), Some(last)) =
                (unit(&event["first_unit"]), unit(&event["last_unit"]))
        {
            add(
                hits,
                Hit {
                    first,
                    last,
                    score: 0.75 + share * 0.2,
                    found_by: vec!["memory"],
                    label: event["title"].as_str().map(str::to_owned),
                },
            );
        }
    }
    let events: BTreeMap<&str, &Value> = memory["events"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|event| Some((event["event_id"].as_str()?, event)))
        .collect();
    linked_events(memory, &events, terms, hits);
    for entity in memory["entities"].as_array().into_iter().flatten() {
        let names: Vec<&str> = std::iter::once(&entity["name"])
            .chain(entity["aliases"].as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .collect();
        if !names.iter().any(|name| {
            overlap(terms, name) >= 0.5 || overlap(&words(name), &terms.join(" ")) >= 1.0
        }) {
            continue;
        }
        for id in entity["events"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let Some(event) = events.get(id) else {
                continue;
            };
            if let (Some(first), Some(last)) =
                (unit(&event["first_unit"]), unit(&event["last_unit"]))
            {
                add(
                    hits,
                    Hit {
                        first,
                        last,
                        score: 0.7,
                        found_by: vec!["memory"],
                        label: event["title"].as_str().map(str::to_owned),
                    },
                );
            }
        }
    }
    for chapter in memory["chapters"].as_array().into_iter().flatten() {
        let text = format!(
            "{} {}",
            chapter["title"].as_str().unwrap_or(""),
            chapter["summary"].as_str().unwrap_or("")
        );
        let share = overlap(terms, &text);
        if share >= 0.6
            && let (Some(first), Some(last)) =
                (unit(&chapter["first_unit"]), unit(&chapter["last_unit"]))
        {
            add(
                hits,
                Hit {
                    first,
                    last,
                    score: 0.6 + share * 0.1,
                    found_by: vec!["memory"],
                    label: chapter["title"].as_str().map(|t| format!("Chapter: {t}")),
                },
            );
        }
    }
}

fn linked_events(
    memory: &Value,
    events: &BTreeMap<&str, &Value>,
    terms: &[String],
    hits: &mut BTreeMap<(u64, u64), Hit>,
) {
    for link in memory["links"].as_array().into_iter().flatten() {
        let description = format!(
            "{} {}",
            link["kind"].as_str().unwrap_or("").replace('_', " "),
            link["note"].as_str().unwrap_or("")
        );
        if overlap(terms, &description) < 0.5 {
            continue;
        }
        for endpoint in ["from_event", "to_event"] {
            if let Some(event) = link[endpoint].as_str().and_then(|id| events.get(id))
                && let (Some(first), Some(last)) =
                    (unit(&event["first_unit"]), unit(&event["last_unit"]))
            {
                add(
                    hits,
                    Hit {
                        first,
                        last,
                        score: 0.75,
                        found_by: vec!["link"],
                        label: link["note"].as_str().map(str::to_owned),
                    },
                );
            }
        }
    }
}

/// IEEE 754 half precision to single, for the stored vectors.
fn f16_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15) << 31;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x3ff);
    let value = match (exponent, mantissa) {
        (0, 0) => sign,
        (0, _) => {
            // Subnormal: normalise it.
            let mut e = 127 - 15 + 1;
            let mut m = mantissa;
            while m & 0x400 == 0 {
                m <<= 1;
                e -= 1;
            }
            sign | (e << 23) | ((m & 0x3ff) << 13)
        }
        (0x1f, 0) => sign | 0x7f80_0000,
        (0x1f, _) => sign | 0x7fc0_0000,
        _ => sign | ((exponent + 127 - 15) << 23) | (mantissa << 13),
    };
    f32::from_bits(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_floats_read_back() {
        assert!((f16_to_f32(0x3c00) - 1.0).abs() < f32::EPSILON);
        assert!((f16_to_f32(0xbc00) + 1.0).abs() < f32::EPSILON);
        assert!((f16_to_f32(0x3800) - 0.5).abs() < f32::EPSILON);
        assert!(f16_to_f32(0x0001) > 0.0);
    }

    #[test]
    fn repeated_evidence_and_question_words_do_not_inflate_relevance() {
        assert_eq!(words("Where do they talk about pricing?"), ["pricing"]);
        let mut hits = BTreeMap::new();
        let memory = json!({"events":[{"event_id":"m1","first_unit":"s0","last_unit":"s1","title":"Pricing"}],
            "entities":[{"name":"Pricing","events":["m1"]}],"links":[]});
        remembered(&memory, &words("pricing"), &mut hits);
        assert_eq!(hits[&(0, 1)].found_by, ["memory"]);
        assert!(hits[&(0, 1)].relevance() < 1.0);
    }

    #[test]
    fn filters_require_overlapping_evidence_and_links_return_both_endpoints() {
        let memory = json!({"chapters":[{"chapter_id":"c1","first_unit":"s0","last_unit":"s2"}],
            "events":[{"event_id":"m1","first_unit":"s1","last_unit":"s2","mood":"funny"},
                      {"event_id":"m2","first_unit":"s10","last_unit":"s10"}],
            "links":[{"kind":"callback","note":"pricing example","from_event":"m1","to_event":"m2"}]});
        let mut hits = BTreeMap::new();
        remembered(&memory, &words("pricing"), &mut hits);
        let filters = Filters {
            speaker: Some("A".into()),
            mood: Some("funny".into()),
            chapter: Some("c1".into()),
            modality: Some("laughter".into()),
        };
        let speakers = BTreeMap::from([(1, "A"), (10, "B")]);
        let laugh = json!({"kind":"laughter","start_ticks":100,"end_ticks":200});
        assert!(filters.accepts(&hits[&(1, 2)], &memory, &speakers, &[&laugh], 50, 150));
        assert!(!filters.accepts(&hits[&(10, 10)], &memory, &speakers, &[&laugh], 200, 300));
    }

    #[test]
    fn a_moment_found_two_ways_outranks_one_found_once() {
        let mut hits = BTreeMap::new();
        let memory = json!({
            "events": [
                {"event_id": "m1", "title": "What the subscription costs", "first_unit": "s3", "last_unit": "s5"},
                {"event_id": "m2", "title": "A hike at dawn", "first_unit": "s9", "last_unit": "s9"},
            ],
            "entities": [], "chapters": [],
        });
        remembered(&memory, &words("subscription costs"), &mut hits);
        add(
            &mut hits,
            Hit {
                first: 3,
                last: 5,
                score: 0.8,
                found_by: vec!["meaning"],
                label: None,
            },
        );
        add(
            &mut hits,
            Hit {
                first: 9,
                last: 9,
                score: 0.85,
                found_by: vec!["meaning"],
                label: None,
            },
        );
        let both = &hits[&(3, 5)];
        assert!(both.relevance() > hits[&(9, 9)].relevance());
        assert_eq!(both.found_by, ["memory", "meaning"]);
    }
}
