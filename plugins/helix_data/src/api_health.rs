//! Optional remote health checks against the Helix standardization API (port 6800).
//!
//! All checks gracefully return `None` or empty results when the API is
//! unavailable. Uses `curl` via `std::process::Command` to avoid adding
//! an HTTP client dependency.

use crate::registries::HelixRegistries;
use std::path::Path;
use std::process::Command;

const API_BASE: &str = "http://localhost:6800";
const TIMEOUT_SECS: &str = "2";

/// Summary of the standardization API's health endpoint.
pub struct ApiHealthReport {
    pub connected: bool,
    pub entities_remote: usize,
    pub entities_local: usize,
    pub data_age_minutes: u64,
    pub version: String,
}

/// Comparison of local TOML file age vs remote data age.
pub struct FreshnessReport {
    pub local_age_minutes: u64,
    pub remote_age_minutes: u64,
    pub needs_refresh: bool,
}

/// Check the standardization API health endpoint.
///
/// Returns `None` if the API is unreachable, times out, or returns
/// unparseable data. Never blocks longer than 2 seconds.
pub fn check_api_health() -> Option<ApiHealthReport> {
    let url = format!("{}/health", API_BASE);
    let output = Command::new("curl")
        .args(["-sf", "--max-time", TIMEOUT_SECS, &url])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let data = json.get("data")?;

    Some(ApiHealthReport {
        connected: true,
        entities_remote: data.get("entities_loaded")?.as_u64()? as usize,
        entities_local: 0, // filled by caller
        data_age_minutes: data.get("data_age_minutes")?.as_u64()?,
        version: data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
    })
}

/// Compare local TOML file freshness against the remote API.
///
/// Uses `abilities.toml` mtime as a proxy for the whole dataset.
/// Returns a report with `needs_refresh = true` when local files
/// are older than 60 minutes.
pub fn check_data_freshness(helix3d_dir: &Path) -> FreshnessReport {
    let local_age = std::fs::metadata(helix3d_dir.join("abilities.toml"))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs() / 60)
        .unwrap_or(0);

    let remote_age = check_api_health().map(|h| h.data_age_minutes).unwrap_or(0);

    FreshnessReport {
        local_age_minutes: local_age,
        remote_age_minutes: remote_age,
        needs_refresh: local_age > 60,
    }
}

/// Validate a sample entity against the remote `/validate` endpoint.
///
/// Sends the first ability from the local registries. Returns a list of
/// issue strings (empty means valid or API unavailable).
pub fn validate_sample_against_api(registries: &HelixRegistries) -> Vec<String> {
    let Some((id, ability)) = registries.abilities.iter().next() else {
        return Vec::new();
    };

    let ability_json = match serde_json::to_value(ability) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let payload = serde_json::json!({
        "type": "abilities",
        "data": ability_json
    });

    let payload_str = payload.to_string();
    let url = format!("{}/validate", API_BASE);

    let output = match Command::new("curl")
        .args([
            "-sf",
            "--max-time",
            TIMEOUT_SECS,
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload_str,
            &url,
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let data = match json.get("data") {
        Some(d) => d,
        None => return Vec::new(),
    };

    let valid = data.get("valid").and_then(|v| v.as_bool()).unwrap_or(true);

    if valid {
        return Vec::new();
    }

    let mut issues = Vec::new();
    if let Some(errors) = data.get("errors").and_then(|e| e.as_array()) {
        for err in errors {
            if let Some(msg) = err.as_str() {
                issues.push(format!("Remote validation ({}): {}", id, msg));
            }
        }
    }

    if issues.is_empty() {
        issues.push(format!(
            "Remote validation ({}): entity failed validation (no details)",
            id
        ));
    }

    issues
}

/// Parse an API health JSON string into an `ApiHealthReport`.
///
/// Exposed for testing without a live API.
pub fn parse_health_response(json_str: &str) -> Option<ApiHealthReport> {
    let json: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let data = json.get("data")?;

    Some(ApiHealthReport {
        connected: true,
        entities_remote: data.get("entities_loaded")?.as_u64()? as usize,
        entities_local: 0,
        data_age_minutes: data.get("data_age_minutes")?.as_u64()?,
        version: data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
    })
}

/// Parse a validate response JSON string into issue strings.
///
/// Exposed for testing without a live API.
pub fn parse_validate_response(json_str: &str, sample_id: &str) -> Vec<String> {
    let json: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let data = match json.get("data") {
        Some(d) => d,
        None => return Vec::new(),
    };

    let valid = data.get("valid").and_then(|v| v.as_bool()).unwrap_or(true);

    if valid {
        return Vec::new();
    }

    let mut issues = Vec::new();
    if let Some(errors) = data.get("errors").and_then(|e| e.as_array()) {
        for err in errors {
            if let Some(msg) = err.as_str() {
                issues.push(format!("Remote validation ({}): {}", sample_id, msg));
            }
        }
    }

    if issues.is_empty() {
        issues.push(format!(
            "Remote validation ({}): entity failed validation (no details)",
            sample_id
        ));
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_health_response_valid() {
        let json = r#"{
            "data": {
                "status": "ok",
                "entities_loaded": 8802,
                "categories_loaded": 321,
                "data_age_minutes": 5,
                "uptime_seconds": 1234
            }
        }"#;

        let report = parse_health_response(json).unwrap();
        assert!(report.connected);
        assert_eq!(report.entities_remote, 8802);
        assert_eq!(report.data_age_minutes, 5);
        assert_eq!(report.version, "ok");
    }

    #[test]
    fn parse_health_response_missing_field_returns_none() {
        let json = r#"{"data": {"status": "ok"}}"#;
        assert!(parse_health_response(json).is_none());
    }

    #[test]
    fn parse_health_response_invalid_json_returns_none() {
        assert!(parse_health_response("not json").is_none());
    }

    #[test]
    fn parse_health_response_empty_returns_none() {
        assert!(parse_health_response("").is_none());
    }

    #[test]
    fn parse_validate_response_valid_entity() {
        let json = r#"{"data": {"valid": true, "errors": []}}"#;
        let issues = parse_validate_response(json, "fireball");
        assert!(issues.is_empty());
    }

    #[test]
    fn parse_validate_response_invalid_entity() {
        let json = r#"{"data": {"valid": false, "errors": ["missing field: damage"]}}"#;
        let issues = parse_validate_response(json, "fireball");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("missing field: damage"));
        assert!(issues[0].contains("fireball"));
    }

    #[test]
    fn parse_validate_response_invalid_no_details() {
        let json = r#"{"data": {"valid": false}}"#;
        let issues = parse_validate_response(json, "test");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("no details"));
    }

    #[test]
    fn parse_validate_response_garbage_returns_empty() {
        let issues = parse_validate_response("not json", "test");
        assert!(issues.is_empty());
    }
}
