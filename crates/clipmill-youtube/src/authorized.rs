//! Credential-backed facade: the daemon persists operation identity, never a
//! bearer token or resumable session URL. A session is saved before media is sent.
use crate::{
    Channel, DesktopClient, Error, Metadata, Token, UploadReply, VideoStatus, YouTube, refresh,
    vault,
};
use serde::{Deserialize, Serialize};

static REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Serialize, Deserialize)]
struct Credentials {
    client: DesktopClient,
    token: Token,
    received_at: u64,
}

/// Contains an opaque upload identity, never the provider's bearer-like URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionHandle {
    upload_id: String,
}

#[derive(Clone, Debug)]
pub struct AuthorizedClient {
    connection_id: String,
}

fn valid_key(value: &str) -> Result<(), Error> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(Error::Invalid("Invalid connection or upload identity"));
    }
    Ok(())
}
fn credential_key(id: &str) -> Result<String, Error> {
    valid_key(id)?;
    Ok(format!("connection:{id}"))
}
fn session_key(id: &str) -> Result<String, Error> {
    valid_key(id)?;
    Ok(format!("session:{id}"))
}
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |v| v.as_secs())
}

pub async fn store_connection(
    id: &str,
    client: &DesktopClient,
    token: &Token,
) -> Result<(), Error> {
    let _guard = REFRESH_LOCK.lock().await;
    token.validate(true)?;
    let saved = Credentials {
        client: client.clone(),
        token: token.clone(),
        received_at: now(),
    };
    vault::put(
        &credential_key(id)?,
        &serde_json::to_vec(&saved).map_err(|_| Error::Protocol)?,
    )
}
pub async fn forget_connection(id: &str) -> Result<(), Error> {
    // Wait for an in-flight refresh before deleting. Otherwise its later save
    // could recreate a credential that the user just disconnected.
    let _guard = REFRESH_LOCK.lock().await;
    vault::delete(&credential_key(id)?)
}

pub fn existing_session(upload_id: &str) -> Result<Option<SessionHandle>, Error> {
    match vault::optional(&session_key(upload_id)?)? {
        Some(bytes) => {
            let url = std::str::from_utf8(&bytes).map_err(|_| Error::Protocol)?;
            crate::validate_session(url)?;
            Ok(Some(SessionHandle {
                upload_id: upload_id.to_owned(),
            }))
        }
        None => Ok(None),
    }
}

/// Only call after the daemon has durably authorized replacing an expired
/// incomplete session and proved no final range could have been sent.
pub fn forget_session(upload_id: &str) -> Result<(), Error> {
    vault::delete(&session_key(upload_id)?)
}
impl SessionHandle {
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
    fn url(&self) -> Result<String, Error> {
        let url = String::from_utf8(vault::get(&session_key(&self.upload_id)?)?)
            .map_err(|_| Error::Protocol)?;
        crate::validate_session(&url)?;
        Ok(url)
    }
}

impl AuthorizedClient {
    pub async fn load(connection_id: &str) -> Result<Self, Error> {
        valid_key(connection_id)?;
        let client = Self {
            connection_id: connection_id.to_owned(),
        };
        client.api().await?;
        Ok(client)
    }
    async fn api(&self) -> Result<YouTube, Error> {
        let _guard = REFRESH_LOCK.lock().await;
        let key = credential_key(&self.connection_id)?;
        let mut saved: Credentials =
            serde_json::from_slice(&vault::get(&key)?).map_err(|_| Error::CredentialStore)?;
        saved.token.validate(true)?;
        if now().saturating_add(60) >= saved.received_at.saturating_add(saved.token.expires_in) {
            saved.token = refresh(&saved.client, &saved.token).await?;
            saved.received_at = now();
            vault::put(
                &key,
                &serde_json::to_vec(&saved).map_err(|_| Error::Protocol)?,
            )?;
        }
        YouTube::new(&saved.token)
    }
    pub async fn channel(&self) -> Result<Channel, Error> {
        self.api().await?.channel().await
    }
    pub fn existing_session(&self, id: &str) -> Result<Option<SessionHandle>, Error> {
        existing_session(id)
    }
    pub async fn begin_private(
        &self,
        upload_id: &str,
        metadata: &Metadata,
        total: u64,
    ) -> Result<SessionHandle, Error> {
        // An existing session is always queried; retries never create a second
        // insert simply because the daemon lost its last SQLite acknowledgement.
        if let Some(handle) = existing_session(upload_id)? {
            return Ok(handle);
        }
        let key = session_key(upload_id)?;
        let session = self.api().await?.begin(metadata, total).await?;
        vault::put(&key, session.as_bytes())?;
        Ok(SessionHandle {
            upload_id: upload_id.to_owned(),
        })
    }
    pub async fn position(
        &self,
        session: &SessionHandle,
        total: u64,
    ) -> Result<UploadReply, Error> {
        self.api().await?.position(&session.url()?, total).await
    }
    pub async fn chunk(
        &self,
        session: &SessionHandle,
        start: u64,
        total: u64,
        bytes: Vec<u8>,
    ) -> Result<UploadReply, Error> {
        self.api()
            .await?
            .chunk(&session.url()?, start, total, bytes)
            .await
    }
    pub async fn video(&self, id: &str, channel: &str) -> Result<VideoStatus, Error> {
        self.api().await?.video(id, channel).await
    }
    pub async fn publish(&self, id: &str, channel: &str) -> Result<VideoStatus, Error> {
        self.api().await?.publish(id, channel).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opaque_id_cannot_select_another_credential_namespace() {
        for id in ["", "../other", "connection:other", "a\nsecret"] {
            assert!(valid_key(id).is_err());
        }
        assert!(valid_key("upload_01M2ABC").is_ok());
    }
}
