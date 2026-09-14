// XYRemote: report remote-control session records to the XYRemote portal for
// audit logging. Intentionally minimal — all storage, logic and UI live
// server-side. The ingest key is injected at build time via the
// XYREMOTE_INGEST_KEY environment variable; builds without it do nothing.

use chrono::{DateTime, Utc};
use std::time::SystemTime;

const REPORT_URL: &str = "https://xyremote.com/api/v1/sessions";

pub fn conn_type_label(t: i32) -> &'static str {
    match t {
        1 => "file-transfer",
        2 => "port-forward",
        3 => "view-camera",
        4 => "terminal",
        _ => "remote",
    }
}

fn to_rfc3339(t: SystemTime) -> String {
    let secs = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    DateTime::<Utc>::from_timestamp(secs, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

// Fire-and-forget report of one finished remote-control session, sent from the
// controlled (host) side. Runs on a detached thread so it never blocks the
// connection teardown.
pub fn report_session(
    controller_id: String,
    controlled_id: String,
    conn_type: String,
    started_at: SystemTime,
    ended_at: SystemTime,
) {
    let key = match option_env!("XYREMOTE_INGEST_KEY") {
        Some(k) if !k.is_empty() => k.to_string(),
        _ => return,
    };
    let started = to_rfc3339(started_at);
    let ended = to_rfc3339(ended_at);
    let version = crate::VERSION.to_string();
    std::thread::spawn(move || {
        let body = serde_json::json!({
            "controller_id": controller_id,
            "controlled_id": controlled_id,
            "connection_type": conn_type,
            "started_at": started,
            "ended_at": ended,
            "reporter_role": "controlled",
            "client_version": version,
        });
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            let _ = client
                .post(REPORT_URL)
                .header("X-XYRemote-Key", key)
                .json(&body)
                .send();
        }
    });
}


pub fn apply_support_code(code: String) {
    use hbb_common::config::Config;
    // Set the incoming-connection password to the agent-provided support code and
    // require it (permanent-password auth). Written to config before the --server
    // subprocess starts, so it is picked up on launch.
    Config::set_permanent_password(&code);
    Config::set_option("verification-method".to_owned(), "use-permanent-password".to_owned());
    // Report our id against the code so the support portal can pair the agent.
    report_support(code, Config::get_id());
}

fn report_support(code: String, peer_id: String) {
    let key = match option_env!("XYREMOTE_INGEST_KEY") {
        Some(k) if !k.is_empty() => k.to_string(),
        _ => return,
    };
    std::thread::spawn(move || {
        let body = serde_json::json!({ "code": code, "peer_id": peer_id });
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            let _ = client
                .post("https://xyremote.com/api/v1/support/register")
                .header("X-XYRemote-Key", key)
                .json(&body)
                .send();
        }
    });
}
