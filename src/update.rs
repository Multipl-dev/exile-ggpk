use std::sync::mpsc::{channel, Receiver};
use std::thread;
use serde::Deserialize;
use semver::Version;

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

pub struct UpdateState {
    pub pending: bool,
    pub latest_version: Option<String>,
    pub release_url: Option<String>,
    pub error_msg: Option<String>,
    receiver: Receiver<Result<Option<(String, String)>, String>>,
}

impl UpdateState {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        
        thread::spawn(move || {
            let result = check_update_impl();
            let _ = tx.send(result);
        });

        Self {
            pending: true,
            latest_version: None,
            release_url: None,
            error_msg: None,
            receiver: rx,
        }
    }

    pub fn poll(&mut self) {
        if self.pending {
            if let Ok(result) = self.receiver.try_recv() {
                self.pending = false;
                match result {
                    Ok(Some((ver, url))) => {
                        self.latest_version = Some(ver);
                        self.release_url = Some(url);
                    },
                    Ok(None) => {
                        // No update available
                    },
                    Err(e) => {
                        self.error_msg = Some(e);
                    }
                }
            }
        }
    }
}

fn check_update_impl() -> Result<Option<(String, String)>, String> {
    let current_version_str = env!("CARGO_PKG_VERSION");
    let current_version = Version::parse(current_version_str).map_err(|e| format!("Failed to parse local version: {}", e))?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("ggpk-explorer-update-checker")
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let resp = client.get("https://api.github.com/repos/juddisjudd/ggpk-explorer/releases/latest")
        .send()
        .map_err(|e| format!("Network error: {}", e))?;
        
    if !resp.status().is_success() {
        return Err(format!("GitHub API Error: {}", resp.status()));
    }

    let release: GitHubRelease = resp.json().map_err(|e| format!("Failed to parse JSON: {}", e))?;
    let tag_name = release.tag_name.trim_start_matches('v');
    
    let latest_version = Version::parse(tag_name).map_err(|e| format!("Failed to parse remote tag '{}': {}", release.tag_name, e))?;
    
    if latest_version > current_version {
        return Ok(Some((release.tag_name, release.html_url)));
    }

    Ok(None)
}

