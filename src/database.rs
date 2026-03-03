use std::error::Error;

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_json::Value;
use base64::engine::general_purpose;
use base64::Engine as _;

/// Download a file from a repository's contents API and return its decoded text content.
///
/// - `owner`, `repo`, `path` identify the file in the repo.
/// - `branch` is an optional branch/ref name (None -> default branch).
/// - `token` is an optional GitHub token; if provided, it's used for authenticated requests.
pub fn download(
    owner: &str,
    repo: &str,
    path: &str,
    branch: Option<&str>,
    token: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let mut url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
    if let Some(br) = branch {
        url.push_str(&format!("?ref={}", br));
    }

    let mut req = client
        .get(&url)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "tic-tac-go");

    if let Some(t) = token {
        req = req.header(AUTHORIZATION, format!("token {}", t));
    }

    let resp = req.send()?;

    match resp.status() {
        reqwest::StatusCode::OK => {
            let json: Value = resp.json()?;
            let content = json["content"].as_str().ok_or("missing 'content' in response")?;
            let encoding = json["encoding"].as_str().unwrap_or("base64");
            if encoding != "base64" {
                return Err(format!("unsupported encoding: {}", encoding).into());
            }
            let cleaned = content.replace('\n', "");
            let bytes = general_purpose::STANDARD.decode(cleaned)?;
            Ok(String::from_utf8(bytes)?)
        }
        reqwest::StatusCode::NOT_FOUND => Err("file not found".into()),
        s => Err(format!("unexpected HTTP status: {}", s).into()),
    }
}

/// Create or update a file in a repository using the Contents API.
/// Returns the blob SHA of the written content on success.
pub fn upload(
    owner: &str,
    repo: &str,
    path: &str,
    branch: Option<&str>,
    token: &str,
    json_content: &str,
    commit_message: &str,
) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    // If no token provided, attempt to commit & push using the user's local git
    // credentials (SSH keys / credential helper) so users don't need to set up
    // a GITHUB_TOKEN for simple pushes from their machine.
    if token.is_empty() {
        // Add the path to the index
        let add_status = std::process::Command::new("git")
            .args(&["add", path])
            .status()?;
        if !add_status.success() {
            return Err(format!("git add failed with exit code: {}", add_status.code().unwrap_or(-1)).into());
        }

        // Check if anything is staged for this path
        let diff_out = std::process::Command::new("git")
            .args(&["diff", "--staged", "--name-only", "--", path])
            .output()?;
        if !diff_out.status.success() {
            return Err(format!("git diff --staged failed with exit code: {}", diff_out.status.code().unwrap_or(-1)).into());
        }

        if diff_out.stdout.is_empty() {
            // Nothing to commit; try to push current branch (in case there are local commits to push)
            let push_status = std::process::Command::new("git")
                .args(&["push", "origin", "HEAD"]) 
                .status()?;
            if !push_status.success() {
                return Err(format!("git push failed with exit code: {}", push_status.code().unwrap_or(-1)).into());
            }
            return Ok(String::new());
        }

        // Commit staged changes for this path
        let commit_status = std::process::Command::new("git")
            .args(&["commit", "-m", commit_message, "--", path])
            .status()?;
        if !commit_status.success() {
            return Err(format!("git commit failed with exit code: {}", commit_status.code().unwrap_or(-1)).into());
        }

        // Push the commit
        let push_status = std::process::Command::new("git")
            .args(&["push", "origin", "HEAD"]) 
            .status()?;
        if !push_status.success() {
            return Err(format!("git push failed with exit code: {}", push_status.code().unwrap_or(-1)).into());
        }

        return Ok(String::new());
    }

    // Try to GET the file to obtain its current sha (if any)
    let mut get_url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
    if let Some(br) = branch {
        get_url.push_str(&format!("?ref={}", br));
    }

    let mut get_req = client
        .get(&get_url)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "tic-tac-go");
    if !token.is_empty() {
        get_req = get_req.header(AUTHORIZATION, format!("token {}", token));
    }

    let sha = match get_req.send()? {
        resp if resp.status() == reqwest::StatusCode::OK => {
            let v: Value = resp.json()?;
            v["sha"].as_str().map(|s| s.to_string())
        }
        _ => None,
    };

    let put_url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);

    let mut body = serde_json::Map::new();
    body.insert(
        "message".to_string(),
        Value::String(commit_message.to_string()),
    );
    body.insert(
        "content".to_string(),
        Value::String(general_purpose::STANDARD.encode(json_content)),
    );
    if let Some(br) = branch {
        body.insert("branch".to_string(), Value::String(br.to_string()));
    }
    if let Some(s) = sha {
        body.insert("sha".to_string(), Value::String(s));
    }

    let mut put_req = client
        .put(&put_url)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "tic-tac-go")
        .json(&body);
    if !token.is_empty() {
        put_req = put_req.header(AUTHORIZATION, format!("token {}", token));
    }

    let resp = put_req.send()?;
    match resp.status() {
        reqwest::StatusCode::CREATED | reqwest::StatusCode::OK => {
            let v: Value = resp.json()?;
            if let Some(sha) = v
                .get("content")
                .and_then(|c| c.get("sha"))
                .and_then(|s| s.as_str())
            {
                Ok(sha.to_string())
            } else {
                Ok(String::new())
            }
        }
        s => {
            let text = resp.text().unwrap_or_default();
            Err(format!("failed to create/update file: {} - {}", s, text).into())
        }
    }
}

// Backward-compatible thin wrappers for older function names (no-op on parameters)
#[allow(dead_code)]
pub fn upload_puzzle(json: String) -> Result<String, Box<dyn Error>> {
    // Caller should supply repo info; this wrapper is kept for compatibility and simply returns an error.
    Err("upload_puzzle wrapper not implemented; use upload(owner, repo, path, branch, token, json_content, commit_message)".into())
}

#[allow(dead_code)]
pub fn download_puzzle() -> Result<String, Box<dyn Error>> {
    Err("download_puzzle wrapper not implemented; use download(owner, repo, path, branch, token)".into())
}
