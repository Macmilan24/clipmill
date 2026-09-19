use crate::{Error, checked, client, json};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::{Duration, timeout},
};
use url::Url;

pub(crate) async fn token_response(response: reqwest::Response) -> Result<Token, Error> {
    let status = response.status().as_u16();
    if matches!(status, 400 | 401 | 403) {
        #[derive(Deserialize)]
        struct Refusal {
            error: String,
        }
        let refusal: Refusal = json(response).await?;
        return match refusal.error.as_str() {
            "invalid_grant" | "invalid_client" | "unauthorized_client" | "access_denied" => {
                Err(Error::Authorization)
            }
            _ => Err(Error::Provider(status)),
        };
    }
    json(checked(response).await?).await
}

pub const SCOPE: &str = "https://www.googleapis.com/auth/youtube.force-ssl";

/// Desktop configuration is imported in the backend and stored in Keychain.
#[derive(Clone, Deserialize, Serialize)]
pub struct DesktopClient {
    pub client_id: String,
    pub client_secret: Option<String>,
}
impl std::fmt::Debug for DesktopClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DesktopClient([redacted])")
    }
}
impl DesktopClient {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        #[derive(Deserialize)]
        struct File {
            installed: DesktopClient,
        }
        if bytes.len() > 64 * 1024 {
            return Err(Error::Invalid("OAuth configuration is too large"));
        }
        let file: File = serde_json::from_slice(bytes)
            .map_err(|_| Error::Invalid("Choose a Google Desktop app OAuth JSON file"))?;
        let id = &file.installed.client_id;
        if !id.ends_with(".apps.googleusercontent.com")
            || id.len() <= 27
            || id.len() > 300
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_'))
            || file
                .installed
                .client_secret
                .as_ref()
                .is_some_and(|s| s.is_empty() || s.len() > 4096 || s.chars().any(char::is_control))
        {
            return Err(Error::Invalid(
                "The file does not contain a valid Desktop app client ID",
            ));
        }
        Ok(file.installed)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    #[serde(default)]
    pub scope: String,
    pub token_type: String,
}
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Token([redacted])")
    }
}
impl Token {
    pub(crate) fn validate(&self, require_refresh: bool) -> Result<(), Error> {
        let valid_secret =
            |s: &str| !s.is_empty() && s.len() <= 16384 && !s.chars().any(char::is_control);
        if !valid_secret(&self.access_token)
            || !self.token_type.eq_ignore_ascii_case("bearer")
            || self.expires_in == 0
            || self.expires_in > 86400
            || !self.scope.split_whitespace().any(|s| s == SCOPE)
            || self
                .refresh_token
                .as_ref()
                .is_some_and(|s| !valid_secret(s))
            || (require_refresh && self.refresh_token.is_none())
        {
            return Err(Error::Authorization);
        }
        Ok(())
    }
}

pub struct PendingAuthorization {
    listener: TcpListener,
    state: String,
    verifier: String,
    redirect: String,
}
impl std::fmt::Debug for PendingAuthorization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingAuthorization([redacted])")
    }
}

fn nonce() -> Result<String, Error> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|_| Error::Invalid("Secure random generator unavailable"))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub async fn authorize(config: &DesktopClient) -> Result<(String, PendingAuthorization), Error> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| Error::Invalid("Cannot open the local Google sign-in callback"))?;
    let port = listener.local_addr().map_err(|_| Error::Protocol)?.port();
    let state = nonce()?;
    let verifier = nonce()?;
    let redirect = format!("http://127.0.0.1:{port}/oauth/callback");
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let mut url =
        Url::parse("https://accounts.google.com/o/oauth2/v2/auth").map_err(|_| Error::Protocol)?;
    url.query_pairs_mut().extend_pairs([
        ("client_id", config.client_id.as_str()),
        ("redirect_uri", redirect.as_str()),
        ("response_type", "code"),
        ("scope", SCOPE),
        ("state", state.as_str()),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("access_type", "offline"),
        ("prompt", "consent"),
    ]);
    Ok((
        url.into(),
        PendingAuthorization {
            listener,
            state,
            verifier,
            redirect,
        },
    ))
}

pub(crate) fn callback(request: &str, expected_state: &str) -> Result<String, Error> {
    let line = request.lines().next().ok_or(Error::Authorization)?;
    let mut parts = line.split_whitespace();
    if parts.next() != Some("GET") {
        return Err(Error::Authorization);
    }
    let target = parts.next().ok_or(Error::Authorization)?;
    if !target.starts_with("/oauth/callback?") {
        return Err(Error::Authorization);
    }
    let url = Url::parse(&format!("http://127.0.0.1{target}")).map_err(|_| Error::Authorization)?;
    let params: Vec<_> = url.query_pairs().collect();
    let one = |name: &str| {
        let values: Vec<_> = params.iter().filter(|(key, _)| key == name).collect();
        (values.len() == 1).then(|| values[0].1.to_string())
    };
    if one("state").as_deref() != Some(expected_state) || one("error").is_some() {
        return Err(Error::Authorization);
    }
    one("code")
        .filter(|code| !code.is_empty() && code.len() <= 8192)
        .ok_or(Error::Authorization)
}

fn denied_callback(request: &str, expected_state: &str) -> bool {
    let Some(target) = request
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("GET "))
        .and_then(|line| line.strip_suffix(" HTTP/1.1"))
    else {
        return false;
    };
    if !target.starts_with("/oauth/callback?") {
        return false;
    }
    let Ok(url) = Url::parse(&format!("http://127.0.0.1{target}")) else {
        return false;
    };
    let states: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .collect();
    let errors: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key == "error")
        .map(|(_, value)| value.into_owned())
        .collect();
    states.len() == 1 && states[0] == expected_state && errors.len() == 1 && !errors[0].is_empty()
}

pub async fn complete_authorization(
    config: &DesktopClient,
    pending: PendingAuthorization,
) -> Result<Token, Error> {
    let code = timeout(Duration::from_mins(5), async {
        loop {
            let (mut stream, peer) = pending.listener.accept().await.map_err(|_| Error::Authorization)?;
            if !peer.ip().is_loopback() { continue; }
            let mut bytes = vec![0_u8; 16 * 1024]; let mut size = 0;
            let read = timeout(Duration::from_secs(5), async {
                loop {
                    let count = stream.read(&mut bytes[size..]).await.map_err(|_| Error::Authorization)?;
                    size += count;
                    if bytes[..size].windows(4).any(|x| x == b"\r\n\r\n") { return Ok(()); }
                    if count == 0 || size == bytes.len() { return Err(Error::Authorization); }
                }
            }).await;
            let request = std::str::from_utf8(&bytes[..size]).unwrap_or("");
            let denied = matches!(read, Ok(Ok(()))) && denied_callback(request, &pending.state);
            let result = if matches!(read, Ok(Ok(()))) { callback(request, &pending.state) } else { Err(Error::Authorization) };
            let (status, body) = if result.is_ok() { ("200 OK", "Sign-in received. Return to ClipMill to check your channel connection.") } else { ("400 Bad Request", "This sign-in callback was not accepted. Return to ClipMill.") };
            let response = format!("HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            let _ = timeout(Duration::from_secs(1), stream.write_all(response.as_bytes())).await;
            if denied { return Err(Error::Authorization); }
            if let Ok(code) = result { return Ok(code); }
        }
    }).await.map_err(|_| Error::Invalid("Google sign-in timed out. Connect again."))??;
    let mut fields = vec![
        ("client_id", config.client_id.as_str()),
        ("code", code.as_str()),
        ("code_verifier", pending.verifier.as_str()),
        ("redirect_uri", pending.redirect.as_str()),
        ("grant_type", "authorization_code"),
    ];
    if let Some(secret) = &config.client_secret {
        fields.push(("client_secret", secret));
    }
    let token = token_response(
        client()?
            .post("https://oauth2.googleapis.com/token")
            .form(&fields)
            .send()
            .await
            .map_err(|_| Error::Network)?,
    )
    .await?;
    token.validate(true)?;
    Ok(token)
}

pub async fn refresh(config: &DesktopClient, previous: &Token) -> Result<Token, Error> {
    let refresh_token = previous
        .refresh_token
        .as_deref()
        .ok_or(Error::Authorization)?;
    let mut fields = vec![
        ("client_id", config.client_id.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    if let Some(secret) = &config.client_secret {
        fields.push(("client_secret", secret));
    }
    let mut token = token_response(
        client()?
            .post("https://oauth2.googleapis.com/token")
            .form(&fields)
            .send()
            .await
            .map_err(|_| Error::Network)?,
    )
    .await?;
    if token.refresh_token.is_none() {
        token.refresh_token.clone_from(&previous.refresh_token);
    }
    if token.scope.is_empty() {
        token.scope.clone_from(&previous.scope);
    }
    token.validate(true)?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_rejects_wrong_state_duplicate_code_and_wrong_path() {
        assert!(
            callback(
                "GET /oauth/callback?state=bad&code=secret HTTP/1.1\r\n",
                "good"
            )
            .is_err()
        );
        assert!(
            callback(
                "GET /oauth/callback?state=good&code=a&code=b HTTP/1.1\r\n",
                "good"
            )
            .is_err()
        );
        assert!(callback("GET /else?state=good&code=a HTTP/1.1\r\n", "good").is_err());
        assert_eq!(
            callback("GET /oauth/callback?state=good&code=a HTTP/1.1\r\n", "good")
                .ok()
                .as_deref(),
            Some("a")
        );
    }
}
