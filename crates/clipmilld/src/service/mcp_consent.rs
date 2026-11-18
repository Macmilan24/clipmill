//! Bind an export rights form to its caller and exact operation across transports.
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Value, json};
use sha2::Sha256;

use super::api_v2::{ApiError, ApiResult};

const LIFETIME_SECS: u64 = 300;
static SIGNING_KEY: OnceLock<Result<[u8; 32], String>> = OnceLock::new();

fn random<const SIZE: usize>() -> Result<[u8; SIZE], String> {
    let mut value = [0; SIZE];
    getrandom::fill(&mut value).map_err(|error| error.to_string())?;
    Ok(value)
}

fn signature(prefix: &str, actor: &str, arguments: &Value) -> Result<Hmac<Sha256>, ApiError> {
    let key = SIGNING_KEY
        .get_or_init(random)
        .as_ref()
        .map_err(|_| ApiError::invalid("Cannot authenticate the export confirmation"))?;
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|_| ApiError::invalid("Invalid confirmation key"))?;
    let content =
        serde_json_canonicalizer::to_vec(&("clipmill.export-rights.v1", prefix, actor, arguments))
            .map_err(|_| ApiError::invalid("Invalid export arguments"))?;
    mac.update(&content);
    Ok(mac)
}

pub(super) fn export_consent(actor: &str, args: &Value) -> ApiResult {
    let arguments = &args["arguments"];
    if !arguments.is_object() {
        return Err(ApiError::invalid("Export arguments must be an object"));
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::invalid("Invalid system time"))?
        .as_secs();
    if let Some(state) = args.get("state") {
        let invalid =
            || ApiError::invalid("Export confirmation expired or did not match this request");
        let state = state
            .as_str()
            .filter(|s| s.len() <= 128)
            .ok_or_else(invalid)?;
        let (prefix, digest) = state.rsplit_once('.').ok_or_else(invalid)?;
        let (expiry, _) = prefix.split_once('.').ok_or_else(invalid)?;
        let expiry: u64 = expiry.parse().map_err(|_| invalid())?;
        if !(now..=now + LIFETIME_SECS).contains(&expiry) {
            return Err(invalid());
        }
        let digest = hex::decode(digest).map_err(|_| invalid())?;
        signature(prefix, actor, arguments)?
            .verify_slice(&digest)
            .map_err(|_| invalid())?;
        return Ok(json!({ "state": state }));
    }
    let nonce =
        random::<16>().map_err(|_| ApiError::invalid("Cannot generate export confirmation"))?;
    let prefix = format!("{}.{}", now + LIFETIME_SECS, hex::encode(nonce));
    let digest = signature(&prefix, actor, arguments)?
        .finalize()
        .into_bytes();
    Ok(json!({ "state": format!("{prefix}.{}", hex::encode(digest)) }))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_form_is_bound_to_the_caller_and_export_arguments() {
        let arguments = json!({"doc_id":"doc_1","destination_dir":"chosen"});
        let challenge = export_consent("owner", &json!({"arguments":arguments}))
            .ok()
            .unwrap();
        let answer = json!({"arguments":arguments,"state":challenge["state"]});
        assert!(export_consent("owner", &answer).is_ok());
        assert!(export_consent("other", &answer).is_err());
        let mut changed = answer.clone();
        changed["arguments"]["destination_dir"] = json!("different");
        assert!(export_consent("owner", &changed).is_err());
        changed = answer;
        changed["state"] = json!("0.nonce.wrong");
        assert!(export_consent("owner", &changed).is_err());
    }
}
