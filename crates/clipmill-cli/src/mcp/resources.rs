//! Read-only edit resources are addressed by daemon IDs, never filesystem paths.
use serde_json::{Value, json};

use super::{Engine, engine_result};

const PREFIX: &str = "clipmill://edits/";

pub(super) fn link(result: &mut Value) {
    let Some(id) = result["structuredContent"]["doc_id"].as_str() else {
        return;
    };
    let link = json!({"type":"resource_link","uri":format!("{PREFIX}{id}"),
        "name":id,"title":"ClipMill edit document","mimeType":"application/json"});
    if let Some(content) = result["content"].as_array_mut() {
        content.push(link);
    }
}

pub(super) fn templates() -> Value {
    json!({"resourceTemplates":[{
        "uriTemplate":"clipmill://edits/{doc_id}","name":"edit","title":"ClipMill edit document","mimeType":"application/json"
    }],"ttlMs":60_000,"cacheScope":"private"})
}

pub(super) async fn read(engine: &Engine, params: &Value) -> Result<Value, String> {
    let uri = params["uri"].as_str().ok_or("Resource URI required")?;
    let id = uri
        .strip_prefix(PREFIX)
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 64
                && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        })
        .ok_or("Unknown edit URI")?;
    let doc = engine_result(engine, "read_edit_resource", json!({"doc_id":id})).await?;
    Ok(
        json!({"ttlMs":0,"cacheScope":"private","contents":[{"uri":uri,"mimeType":"application/json","text":doc.to_string()}]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_results_link_back_to_the_readable_edit() {
        let mut result = super::super::tool_result(Ok(json!({"doc_id":"doc_7","revision":1})));
        link(&mut result);
        assert_eq!(result["content"][1]["type"], "resource_link");
        assert_eq!(result["content"][1]["uri"], "clipmill://edits/doc_7");
        assert_eq!(result["structuredContent"]["revision"], 1);
    }
}
