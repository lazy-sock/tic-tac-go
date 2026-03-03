use std::error::Error;
use std::env;

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
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not set; set SUPABASE_URL to your Supabase project URL")?;
    let maybe_key = env::var("SUPABASE_ANON_KEY").ok()
        .or_else(|| env::var("SUPABASE_SERVICE_ROLE_KEY").ok())
        .or_else(|| env::var("SUPABASE_KEY").ok());

    let client = Client::new();
    let url = format!("{}/rest/v1/puzzles", supabase_url.trim_end_matches('/'));

    // parse JSON content to ensure it's valid JSON and to store as jsonb
    let parsed: Value = serde_json::from_str(json_content)?;
    let body = serde_json::json!({
        "file_name": file_name,
        "content": parsed
    });

    let mut req = client.post(&url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "tic-tac-go")
        .header("Prefer", "return=representation")
        .json(&body);

    if let Some(ref k) = maybe_key {
        req = req.header("apikey", k.as_str()).header(AUTHORIZATION, format!("Bearer {}", k));
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

/// Download a puzzle by file_name from Supabase puzzles table.
/// Returns the content JSON as string.
pub fn download(file_name: &str) -> Result<String, Box<dyn Error>> {
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not set; set SUPABASE_URL to your Supabase project URL")?;
    let maybe_key = env::var("SUPABASE_ANON_KEY").ok()
        .or_else(|| env::var("SUPABASE_SERVICE_ROLE_KEY").ok())
        .or_else(|| env::var("SUPABASE_KEY").ok());

    let client = Client::new();
    let url = format!("{}/rest/v1/puzzles", supabase_url.trim_end_matches('/'));
    let query_file = format!("eq.{}", file_name);

    let mut req = client.get(&url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "tic-tac-go")
        .query(&[("file_name", query_file.as_str()), ("select", "content")]);

    if let Some(ref k) = maybe_key {
        req = req.header("apikey", k.as_str()).header(AUTHORIZATION, format!("Bearer {}", k));
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
