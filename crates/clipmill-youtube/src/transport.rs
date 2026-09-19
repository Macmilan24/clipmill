use crate::{Error, Token, checked, client};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Channel {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VideoStatus {
    pub id: String,
    pub channel_id: String,
    pub privacy: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Metadata {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub made_for_kids: bool,
    pub contains_synthetic_media: bool,
}
impl Metadata {
    pub fn validate(&self) -> Result<(), Error> {
        if self.title.trim().is_empty()
            || self.title.chars().count() > 100
            || self.title.contains(['<', '>'])
            || self.title.chars().any(char::is_control)
        {
            return Err(Error::Invalid(
                "Use a title of 1–100 characters without angle brackets",
            ));
        }
        if self.description.len() > 5000
            || self.description.contains(['<', '>'])
            || self
                .description
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t' | '\r'))
        {
            return Err(Error::Invalid(
                "Use a description under 5,000 UTF-8 bytes without angle brackets",
            ));
        }
        let tag_bytes: usize = self
            .tags
            .iter()
            .map(|tag| tag.len() + 1 + if tag.contains(' ') { 2 } else { 0 })
            .sum();
        if tag_bytes > 500
            || self
                .tags
                .iter()
                .any(|tag| tag.trim().is_empty() || tag.chars().any(char::is_control))
        {
            return Err(Error::Invalid(
                "Tags must be nonempty and total at most 500 bytes",
            ));
        }
        Ok(())
    }
    fn body(&self) -> Value {
        json!({"snippet": {"title": self.title, "description": self.description, "tags": self.tags, "categoryId":"22"},
            "status": {"privacyStatus":"private", "selfDeclaredMadeForKids": self.made_for_kids, "containsSyntheticMedia": self.contains_synthetic_media}})
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UploadReply {
    Incomplete {
        acknowledged: u64,
    },
    Complete {
        video_id: String,
        channel_id: String,
        privacy: String,
    },
}

pub struct YouTube {
    http: reqwest::Client,
    token: String,
    #[cfg(test)]
    test_endpoint: Option<Url>,
}
impl std::fmt::Debug for YouTube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("YouTube([redacted])")
    }
}
impl YouTube {
    pub fn new(token: &Token) -> Result<Self, Error> {
        token.validate(false)?;
        Ok(Self {
            http: client()?,
            token: token.access_token.clone(),
            #[cfg(test)]
            test_endpoint: None,
        })
    }
    #[cfg(test)]
    pub(crate) fn with_test_endpoint(token: &Token, base: &str) -> Result<Self, Error> {
        token.validate(false)?;
        let base = Url::parse(base).map_err(|_| Error::Protocol)?;
        if base.scheme() != "http" || base.host_str() != Some("127.0.0.1") {
            return Err(Error::Protocol);
        }
        Ok(Self {
            http: crate::client_builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|_| Error::Network)?,
            token: token.access_token.clone(),
            test_endpoint: Some(base),
        })
    }
    #[cfg_attr(
        not(test),
        expect(
            clippy::unused_self,
            reason = "Only test builds carry a loopback endpoint override"
        )
    )]
    fn url(&self, canonical: &str) -> Result<Url, Error> {
        let url = Url::parse(canonical).map_err(|_| Error::Protocol)?;
        #[cfg(test)]
        if let Some(base) = &self.test_endpoint {
            let mut local = base.clone();
            local.set_path(url.path());
            local.set_query(url.query());
            return Ok(local);
        }
        Ok(url)
    }

    pub async fn channel(&self) -> Result<Channel, Error> {
        let response = self
            .http
            .get(self.url("https://www.googleapis.com/youtube/v3/channels")?)
            .query(&[("part", "snippet"), ("mine", "true")])
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|_| Error::Network)?;
        let value: Value = crate::json(checked(response).await?).await?;
        let items = value["items"].as_array().ok_or(Error::Protocol)?;
        if items.len() != 1 {
            return Err(Error::Invalid(
                "Select one YouTube channel during Google sign-in",
            ));
        }
        Ok(Channel {
            id: field(&items[0], "id")?,
            title: field(&items[0]["snippet"], "title")?,
        })
    }

    /// Persist the returned URI in the credential store before sending bytes.
    pub async fn begin(&self, metadata: &Metadata, bytes: u64) -> Result<String, Error> {
        metadata.validate()?;
        if bytes == 0 {
            return Err(Error::Invalid("The approved video is empty"));
        }
        let response = self
            .http
            .post(self.url("https://www.googleapis.com/upload/youtube/v3/videos")?)
            .query(&[
                ("uploadType", "resumable"),
                ("part", "snippet,status"),
                ("notifySubscribers", "false"),
            ])
            .bearer_auth(&self.token)
            .header("X-Upload-Content-Length", bytes)
            .header("X-Upload-Content-Type", "video/mp4")
            .json(&metadata.body())
            .send()
            .await
            .map_err(|_| Error::Network)?;
        let response = checked(response).await?;
        let session = response
            .headers()
            .get("Location")
            .and_then(|value| value.to_str().ok())
            .ok_or(Error::Protocol)?;
        validate_session(session)?;
        Ok(session.to_owned())
    }

    /// The provider's acknowledged offset is authoritative after any interruption.
    pub async fn position(&self, session: &str, total: u64) -> Result<UploadReply, Error> {
        validate_session(session)?;
        if total == 0 {
            return Err(Error::Protocol);
        }
        let response = self
            .http
            .put(self.url(session)?)
            .bearer_auth(&self.token)
            .header("Content-Length", 0)
            .header("Content-Range", format!("bytes */{total}"))
            .send()
            .await
            .map_err(|_| Error::Network)?;
        upload_reply(response, total).await
    }

    pub async fn chunk(
        &self,
        session: &str,
        start: u64,
        total: u64,
        bytes: Vec<u8>,
    ) -> Result<UploadReply, Error> {
        validate_session(session)?;
        let count = u64::try_from(bytes.len()).map_err(|_| Error::Protocol)?;
        let end = start.checked_add(count).ok_or(Error::Protocol)?;
        if count == 0 || end > total || (end < total && count % (256 * 1024) != 0) {
            return Err(Error::Invalid("Invalid resumable upload range"));
        }
        let response = self
            .http
            .put(self.url(session)?)
            .bearer_auth(&self.token)
            .header("Content-Type", "video/mp4")
            .header(
                "Content-Range",
                format!("bytes {start}-{}/{total}", end - 1),
            )
            .body(bytes)
            .send()
            .await
            .map_err(|_| Error::Network)?;
        upload_reply(response, total).await
    }

    /// Read first, then update only privacy while carrying every mutable status
    /// property back. Repeating a successful explicit Publish is idempotent.
    async fn video_document(&self, video_id: &str, expected_channel: &str) -> Result<Value, Error> {
        validate_video_id(video_id)?;
        let response = self
            .http
            .get(self.url("https://www.googleapis.com/youtube/v3/videos")?)
            .query(&[
                ("part", "snippet,status,processingDetails"),
                ("id", video_id),
            ])
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|_| Error::Network)?;
        let value: Value = crate::json(checked(response).await?).await?;
        let video = value["items"]
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or(Error::Protocol)?;
        if video["id"].as_str() != Some(video_id) {
            return Err(Error::Protocol);
        }
        if video["snippet"]["channelId"].as_str() != Some(expected_channel) {
            return Err(Error::Invalid("This video belongs to a different channel"));
        }
        Ok(video.clone())
    }

    pub async fn video(
        &self,
        video_id: &str,
        expected_channel: &str,
    ) -> Result<VideoStatus, Error> {
        video_status(
            &self.video_document(video_id, expected_channel).await?,
            expected_channel,
        )
    }

    pub async fn publish(
        &self,
        video_id: &str,
        expected_channel: &str,
    ) -> Result<VideoStatus, Error> {
        let video = self.video_document(video_id, expected_channel).await?;
        if video["status"]["privacyStatus"] == "public" {
            return video_status(&video, expected_channel);
        }
        if matches!(
            video["status"]["uploadStatus"].as_str(),
            Some("failed" | "rejected" | "deleted")
        ) {
            return Err(Error::Invalid(
                "YouTube could not process this video. Review its status in YouTube Studio before publishing.",
            ));
        }
        if video["status"]["uploadStatus"] != "processed" {
            return Err(Error::Invalid(
                "YouTube is still processing this video. Try Publish when processing finishes.",
            ));
        }
        let status = public_status(&video["status"])?;
        let etag = field(&video, "etag")?;
        let response = self
            .http
            .put(self.url("https://www.googleapis.com/youtube/v3/videos")?)
            .query(&[("part", "status")])
            .bearer_auth(&self.token)
            .header("If-Match", etag)
            .json(&json!({"id": video_id, "status": status}))
            .send()
            .await
            .map_err(|_| Error::Network)?;
        let updated: Value = crate::json(checked(response).await?).await?;
        if updated["id"].as_str() != Some(video_id)
            || updated["status"]["privacyStatus"] != "public"
        {
            return Err(Error::Invalid(
                "YouTube kept this video private. The API project may require Google's upload compliance audit.",
            ));
        }
        video_status(&updated, expected_channel)
    }
}

fn video_status(video: &Value, channel: &str) -> Result<VideoStatus, Error> {
    let privacy = field(&video["status"], "privacyStatus")?;
    let status = field(&video["status"], "uploadStatus")?;
    if !matches!(privacy.as_str(), "private" | "unlisted" | "public")
        || !matches!(
            status.as_str(),
            "uploaded" | "processed" | "rejected" | "failed" | "deleted"
        )
    {
        return Err(Error::Protocol);
    }
    Ok(VideoStatus {
        id: field(video, "id")?,
        channel_id: channel.to_owned(),
        privacy,
        status,
    })
}

pub(crate) fn public_status(value: &Value) -> Result<Value, Error> {
    let input = value.as_object().ok_or(Error::Protocol)?;
    let mut status = serde_json::Map::new();
    for key in [
        "license",
        "embeddable",
        "publicStatsViewable",
        "selfDeclaredMadeForKids",
        "containsSyntheticMedia",
    ] {
        if let Some(value) = input.get(key) {
            status.insert(key.to_owned(), value.clone());
        }
    }
    status.insert("privacyStatus".into(), json!("public"));
    Ok(Value::Object(status))
}

pub fn validate_session(session: &str) -> Result<(), Error> {
    if session.len() > 8192 || session.chars().any(char::is_control) {
        return Err(Error::Protocol);
    }
    let url = Url::parse(session).map_err(|_| Error::Protocol)?;
    if url.scheme() != "https"
        || url.host_str() != Some("www.googleapis.com")
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.path() != "/upload/youtube/v3/videos"
        || url
            .query_pairs()
            .filter(|(key, value)| key == "upload_id" && !value.is_empty())
            .count()
            != 1
        || url
            .query_pairs()
            .filter(|(key, _)| key == "upload_id")
            .count()
            != 1
    {
        return Err(Error::Protocol);
    }
    Ok(())
}

fn validate_video_id(id: &str) -> Result<(), Error> {
    if id.len() != 11
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(Error::Protocol);
    }
    Ok(())
}

fn field(value: &Value, key: &str) -> Result<String, Error> {
    value[key]
        .as_str()
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
        .ok_or(Error::Protocol)
}

pub(crate) fn acknowledged(range: Option<&str>, total: u64) -> Result<u64, Error> {
    let Some(range) = range else {
        return Ok(0);
    };
    let last: u64 = range
        .strip_prefix("bytes=0-")
        .ok_or(Error::Protocol)?
        .parse()
        .map_err(|_| Error::Protocol)?;
    let next = last
        .checked_add(1)
        .filter(|next| *next <= total)
        .ok_or(Error::Protocol)?;
    Ok(next)
}

pub(crate) async fn upload_reply(
    response: reqwest::Response,
    total: u64,
) -> Result<UploadReply, Error> {
    if response.status().as_u16() == 308 {
        let range = response
            .headers()
            .get("Range")
            .map(|v| v.to_str().map_err(|_| Error::Protocol))
            .transpose()?;
        return Ok(UploadReply::Incomplete {
            acknowledged: acknowledged(range, total)?,
        });
    }
    if matches!(response.status().as_u16(), 404 | 410) {
        return Err(Error::ExpiredSession);
    }
    let value: Value = crate::json(checked(response).await?).await?;
    let video_id = field(&value, "id")?;
    validate_video_id(&video_id)?;
    let privacy = field(&value["status"], "privacyStatus")?;
    if !matches!(privacy.as_str(), "private" | "unlisted" | "public") {
        return Err(Error::Protocol);
    }
    Ok(UploadReply::Complete {
        video_id,
        channel_id: field(&value["snippet"], "channelId")?,
        privacy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_cannot_redirect_credentials_or_media() {
        assert!(
            validate_session("https://www.googleapis.com/upload/youtube/v3/videos?upload_id=abc")
                .is_ok()
        );
        for url in [
            "http://www.googleapis.com/upload/youtube/v3/videos?upload_id=x",
            "https://www.googleapis.com.evil.com/upload/youtube/v3/videos?upload_id=x",
            "https://user@www.googleapis.com/upload/youtube/v3/videos?upload_id=x",
            "https://www.googleapis.com:444/upload/youtube/v3/videos?upload_id=x",
            "https://www.googleapis.com/anything?upload_id=x",
        ] {
            assert!(validate_session(url).is_err());
        }
    }
    #[test]
    fn acknowledged_offsets_require_contiguous_bounded_ranges() {
        assert_eq!(
            acknowledged(Some("bytes=0-262143"), 500_000).ok(),
            Some(262_144)
        );
        assert_eq!(acknowledged(None, 500_000).ok(), Some(0));
        assert!(acknowledged(Some("bytes=10-20"), 500_000).is_err());
        assert!(acknowledged(Some("bytes=0-500000"), 500_000).is_err());
    }
    #[test]
    fn publish_preserves_mutable_status_but_drops_read_only_and_schedule() {
        let old = json!({"privacyStatus":"private", "license":"creativeCommon", "embeddable":false, "selfDeclaredMadeForKids":true, "containsSyntheticMedia":true, "publishAt":"2028-01-01", "uploadStatus":"processed"});
        let new = public_status(&old).unwrap_or_default();
        assert_eq!(new["privacyStatus"], "public");
        assert_eq!(new["license"], "creativeCommon");
        assert_eq!(new["selfDeclaredMadeForKids"], true);
        assert!(new.get("uploadStatus").is_none());
        assert!(new.get("publishAt").is_none());
    }
}
