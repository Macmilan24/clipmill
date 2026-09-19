//! `YouTube` transport. The daemon owns durable intent and calls this only after
//! explicit connection/upload/publish actions. No ambient discovery or retries.

mod authorized;
mod oauth;
#[cfg(test)]
mod protocol_tests;
mod transport;
pub mod vault;

pub use authorized::{
    AuthorizedClient, SessionHandle, existing_session, forget_connection, forget_session,
    store_connection,
};
pub use oauth::SCOPE;
pub use oauth::{
    DesktopClient, PendingAuthorization, Token, authorize, complete_authorization, refresh,
};
pub use transport::{Channel, Metadata, UploadReply, VideoStatus, YouTube, validate_session};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(&'static str),
    #[error("Google could not be reached. Retry when the connection is available.")]
    Network,
    #[error("Google authorization expired or was refused. Reconnect the channel.")]
    Authorization,
    #[error(
        "YouTube refused the request (HTTP {0}). Check the API project and channel permissions."
    )]
    Provider(u16),
    #[error("The upload session expired. Check YouTube Studio before starting another upload.")]
    ExpiredSession,
    #[error("YouTube returned an unexpected response; the upload needs reconciliation.")]
    Protocol,
    #[error("The operating system credential store is unavailable.")]
    CredentialStore,
    #[error("The saved channel credentials are missing. Reconnect the channel.")]
    MissingCredential,
    #[error("YouTube channel connection currently requires macOS Keychain.")]
    UnsupportedPlatform,
}

fn client_builder() -> reqwest::ClientBuilder {
    // Select the supported ring provider explicitly; keep the normal platform
    // certificate verifier and TLS validation. Another installed provider wins.
    let _already_installed = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_mins(2))
}

fn client() -> Result<reqwest::Client, Error> {
    client_builder()
        .https_only(true)
        .build()
        .map_err(|_| Error::Network)
}

async fn checked(response: reqwest::Response) -> Result<reqwest::Response, Error> {
    match response.status().as_u16() {
        200..=299 => Ok(response),
        401 => Err(Error::Authorization),
        code => Err(Error::Provider(code)),
    }
}

/// Cap every provider JSON body, including OAuth, before decoding it. Never
/// include provider bodies or request URLs in errors because they can carry secrets.
async fn json<T: serde::de::DeserializeOwned>(mut response: reqwest::Response) -> Result<T, Error> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| Error::Network)? {
        if bytes.len().saturating_add(chunk.len()) > 1024 * 1024 {
            return Err(Error::Protocol);
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| Error::Protocol)
}
