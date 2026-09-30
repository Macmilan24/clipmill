//! "A new version is out": the daemon asks GitHub, counted by the Local Lock;
//! the shell opens the release's page, and nothing else, in the browser.

use std::{sync::Arc, time::Duration};

use clipmill_contracts::proto::ipc::v1::{self as ipc, request, response};
use serde::Serialize;
use tauri::State;

use crate::DaemonSupervisor;

type Host<'a> = State<'a, Arc<DaemonSupervisor>>;

/// Where releases are published: the workspace's `repository`.
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
/// The daemon gives GitHub twenty seconds; this waits a little longer.
const CHECK_LIMIT: Duration = Duration::from_secs(30);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    current_version: String,
    latest_version: String,
    newer: bool,
}

/// Ask the daemon which release is newest. The renderer asks at most once a
/// day, and only while the person leaves the notice on.
#[tauri::command]
pub async fn check_for_update(supervisor: Host<'_>) -> Result<UpdateView, String> {
    let reply = supervisor
        .client()
        .call_within(
            request::Body::CheckForUpdate(ipc::CheckForUpdateRequest {}),
            CHECK_LIMIT,
        )
        .await
        .map_err(|error| error.to_string())?;
    let response::Body::CheckForUpdate(reply) = reply else {
        return Err("The engine answered something else.".to_owned());
    };
    Ok(UpdateView {
        current_version: reply.current_version,
        latest_version: reply.latest_version,
        newer: reply.newer,
    })
}

/// Open one release's page on GitHub. The renderer names a version, never an
/// address, so nothing it holds can send the browser anywhere else.
#[tauri::command]
pub async fn open_release_page(version: String) -> Result<(), String> {
    let url = release_page(&version).ok_or("That is not a release version.")?;
    crate::youtube::open_browser(&url)
        .await
        .map_err(|_| "The browser could not be opened.".to_owned())
}

fn release_page(version: &str) -> Option<String> {
    let parts = version.split('.').collect::<Vec<_>>();
    let plain = parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty() && part.len() <= 9 && part.bytes().all(|byte| byte.is_ascii_digit())
        });
    plain.then(|| format!("{REPOSITORY}/releases/tag/v{version}"))
}

#[cfg(test)]
mod tests {
    use super::release_page;

    #[test]
    fn only_a_version_becomes_a_page() {
        assert_eq!(
            release_page("0.4.0").as_deref(),
            Some("https://github.com/Macmilan24/clipmill/releases/tag/v0.4.0")
        );
        for refused in [
            "",
            "0.4",
            "0.4.0/../../x",
            "0.4.0?x=1",
            "v0.4.0",
            "0.4.0-rc.1",
        ] {
            assert_eq!(release_page(refused), None, "{refused:?}");
        }
    }
}
