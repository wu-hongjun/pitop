use std::sync::{Arc, Mutex};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_URL: &str = "https://api.github.com/repos/wu-hongjun/pitop/releases/latest";

/// Result of an update check.
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// Check hasn't completed yet.
    Checking,
    /// A newer version is available.
    Available(String),
    /// Already on the latest version.
    UpToDate,
    /// Check failed (network error, no releases, etc.) — silently ignored.
    Failed,
}

/// Shared handle to the update check result.
pub type UpdateHandle = Arc<Mutex<UpdateStatus>>;

/// Spawn a background task that checks GitHub for a newer release.
/// Returns a shared handle that the UI can poll without blocking.
pub fn spawn_check() -> UpdateHandle {
    let handle = Arc::new(Mutex::new(UpdateStatus::Checking));
    let h = handle.clone();

    tokio::spawn(async move {
        let result = check_latest().await;
        if let Ok(mut guard) = h.lock() {
            *guard = result;
        }
    });

    handle
}

async fn check_latest() -> UpdateStatus {
    let output = tokio::process::Command::new("curl")
        .args([
            "-sL",
            "-m",
            "5",
            "-H",
            "Accept: application/vnd.github.v3+json",
            RELEASES_URL,
        ])
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return UpdateStatus::Failed,
    };

    let body = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return UpdateStatus::Failed,
    };

    // Parse the tag_name from JSON without adding a serde_json dependency
    // for this single call. Look for "tag_name": "vX.Y.Z"
    let latest = match parse_tag_name(&body) {
        Some(v) => v,
        None => return UpdateStatus::Failed,
    };

    if is_newer(&latest, CURRENT_VERSION) {
        UpdateStatus::Available(latest)
    } else {
        UpdateStatus::UpToDate
    }
}

/// Extract tag_name value from GitHub API JSON response.
fn parse_tag_name(json: &str) -> Option<String> {
    // Find "tag_name" : "vX.Y.Z"
    let idx = json.find("\"tag_name\"")?;
    let rest = &json[idx..];
    // Find the colon, then the opening quote of the value
    let colon = rest.find(':')?;
    let after_colon = &rest[colon + 1..];
    let open_quote = after_colon.find('"')?;
    let value_start = open_quote + 1;
    let close_quote = after_colon[value_start..].find('"')?;
    let tag = &after_colon[value_start..value_start + close_quote];
    Some(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

/// Compare two semver-like version strings (e.g. "0.2.0" > "0.1.0").
fn is_newer(latest: &str, current: &str) -> bool {
    let parse =
        |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect() };
    let l = parse(latest);
    let c = parse(current);
    l > c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn test_parse_tag_name() {
        let json = r#"{"tag_name": "v0.2.0", "name": "Release 0.2.0"}"#;
        assert_eq!(parse_tag_name(json), Some("0.2.0".to_string()));
    }

    #[test]
    fn test_parse_tag_name_no_v_prefix() {
        let json = r#"{"tag_name": "1.0.0"}"#;
        assert_eq!(parse_tag_name(json), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_tag_name_missing() {
        assert_eq!(parse_tag_name("{}"), None);
        assert_eq!(parse_tag_name(""), None);
    }
}
