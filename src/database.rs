use dotenvy::dotenv;
use std::env;
use std::error::Error;

// Prefer a compile-time embedded SUPABASE_ANON_KEY when available. This allows
// shipping the anon key inside the binary so users don't need to set env vars.
// To embed the key at build time, set the env var when running cargo, for
// example:
//
//   SUPABASE_ANON_KEY=your_anon_key cargo build --release
//
// or use a build script (build.rs) that emits:
//   println!("cargo:rustc-env=SUPABASE_ANON_KEY={}", key);
//
// The compile-time key is preferred; otherwise, runtime .env or environment
// variables SUPABASE_ANON_KEY, SUPABASE_SERVICE_ROLE_KEY or SUPABASE_KEY are
// used. WARNING: Do NOT embed a service_role key into client binaries.
fn get_supabase_key() -> Option<String> {
    if let Some(k) = option_env!("SUPABASE_ANON_KEY") {
        if !k.is_empty() {
            return Some(k.to_string());
        }
    }
    let _ = dotenv().ok();
    env::var("SUPABASE_ANON_KEY")
        .ok()
        .or_else(|| env::var("SUPABASE_SERVICE_ROLE_KEY").ok())
        .or_else(|| env::var("SUPABASE_KEY").ok())
}

fn get_supabase_url() -> Option<String> {
    if let Some(k) = option_env!("SUPABASE_URL") {
        if !k.is_empty() {
            return Some(k.to_string());
        }
    }
    let _ = dotenv().ok();
    env::var("SUPABASE_URL").ok()
}

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_json::Value;

/// Upload a puzzle JSON into Supabase (PostgREST) table 'puzzles'.
///
/// Requires SUPABASE_URL environment variable. Optionally uses SUPABASE_ANON_KEY
/// or SUPABASE_SERVICE_ROLE_KEY/SUPABASE_KEY if set. If no key is set the
/// request will be attempted without authentication (requires your Supabase
/// config to allow unauthenticated inserts).
pub fn upload(file_name: &str, json_content: &str) -> Result<String, Box<dyn Error>> {
    let _ = dotenv().ok();
    let supabase_url = get_supabase_url().unwrap_or(String::from("supabase url not found"));
    let maybe_key = get_supabase_key();

    let client = Client::new();
    let url = format!("{}/rest/v1/puzzles", supabase_url.trim_end_matches('/'));

    // parse JSON content to ensure it's valid JSON and to store as jsonb
    let parsed: Value = serde_json::from_str(json_content)?;
    let body = serde_json::json!({
        "file_name": file_name,
        "content": parsed
    });

    let mut req = client
        .post(&url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "tic-tac-go")
        .header("Prefer", "return=representation")
        .json(&body);

    if let Some(ref k) = maybe_key {
        req = req
            .header("apikey", k.as_str())
            .header(AUTHORIZATION, format!("Bearer {}", k));
    }

    let resp = req.send()?;
    let status = resp.status();
    if status.is_success() {
        let arr: Value = resp.json()?;
        if let Some(first) = arr.get(0) {
            if let Some(id) = first.get("id") {
                return Ok(id.to_string());
            }
        }
        Ok(String::new())
    } else {
        let text = resp.text().unwrap_or_default();
        Err(format!("supabase upload failed: {} - {}", status, text).into())
    }
}

pub fn list_puzzles() -> Result<Vec<(String, Option<u64>)>, Box<dyn Error>> {
    let _ = dotenv().ok();
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not set; set SUPABASE_URL to your Supabase project URL")?;
    let maybe_key = get_supabase_key();

    let client = Client::new();
    let url = format!("{}/rest/v1/puzzles", supabase_url.trim_end_matches('/'));

    let mut req = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "tic-tac-go")
        .query(&[("select", "file_name,created_at")]);

    if let Some(ref k) = maybe_key {
        req = req
            .header("apikey", k.as_str())
            .header(AUTHORIZATION, format!("Bearer {}", k));
    }

    let resp = req.send()?;
    let status = resp.status();
    if status.is_success() {
        let arr: Value = resp.json()?;
        let mut out = Vec::new();
        if let Some(list) = arr.as_array() {
            for item in list {
                if let Some(name) = item.get("file_name").and_then(|v| v.as_str()) {
                    let created_at = item.get("created_at").and_then(|v| {
                        if v.is_number() {
                            v.as_i64().map(|n| n as u64)
                        } else if v.is_string() {
                            v.as_str().and_then(|s| s.parse::<u64>().ok())
                        } else {
                            None
                        }
                    });
                    out.push((name.to_string(), created_at));
                }
            }
        }
        Ok(out)
    } else {
        let text = resp.text().unwrap_or_default();
        Err(format!("supabase list failed: {} - {}", status, text).into())
    }
}

/// Download a puzzle by file_name from Supabase puzzles table.
/// Returns the content JSON as string.
pub fn download(file_name: &str) -> Result<String, Box<dyn Error>> {
    let _ = dotenv().ok();
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not set; set SUPABASE_URL to your Supabase project URL")?;
    let maybe_key = get_supabase_key();

    let client = Client::new();
    let url = format!("{}/rest/v1/puzzles", supabase_url.trim_end_matches('/'));
    let query_file = format!("eq.{}", file_name);

    let mut req = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "tic-tac-go")
        .query(&[("file_name", query_file.as_str()), ("select", "content")]);

    if let Some(ref k) = maybe_key {
        req = req
            .header("apikey", k.as_str())
            .header(AUTHORIZATION, format!("Bearer {}", k));
    }

    let resp = req.send()?;
    match resp.status() {
        reqwest::StatusCode::OK => {
            let arr: Value = resp.json()?;
            if let Some(first) = arr.get(0) {
                if let Some(content) = first.get("content") {
                    return Ok(content.to_string());
                }
            }
            Err("puzzle not found".into())
        }
        s => {
            let text = resp.text().unwrap_or_default();
            Err(format!("supabase download failed: {} - {}", s, text).into())
        }
    }
}
