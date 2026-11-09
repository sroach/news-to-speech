use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Old state format: HashSet of URLs without timestamps
#[derive(Deserialize)]
struct OldState {
    urls: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedState {
    /// URL -> timestamp when it was first processed
    pub urls: HashMap<String, String>,
}

impl ProcessedState {
    pub fn new() -> Self {
        ProcessedState {
            urls: HashMap::new(),
        }
    }

    pub fn is_processed(&self, url: &str) -> bool {
        self.urls.contains_key(url)
    }

    pub fn mark_processed(&mut self, url: String) {
        let now = Local::now().to_rfc3339();
        self.urls.insert(url, now);
    }

    /// Remove entries older than the given cutoff time.
    /// Returns the number of removed entries.
    pub fn prune_old(&mut self, cutoff: DateTime<Local>) -> usize {
        let before = self.urls.len();
        self.urls.retain(|_url, ts| {
            DateTime::parse_from_rfc3339(ts)
                .ok()
                .and_then(|dt| dt.with_timezone(&Local).into())
                .map(|dt: DateTime<Local>| dt >= cutoff)
                .unwrap_or(false)
        });
        before - self.urls.len()
    }
}

pub fn load_state(path: &Path) -> ProcessedState {
    match fs::read_to_string(path) {
        Ok(contents) => {
            // Try the new format first (HashMap with timestamps)
            match serde_json::from_str::<ProcessedState>(&contents) {
                Ok(state) => {
                    tracing::info!("Loaded state with {} processed URLs", state.urls.len());
                    state
                }
                Err(_) => {
                    // Fall back to the old format (HashSet of URLs without timestamps)
                    match serde_json::from_str::<OldState>(&contents) {
                        Ok(old_state) => {
                            let now = Local::now().to_rfc3339();
                            let mut state = ProcessedState::new();
                            for url in old_state.urls {
                                state.urls.insert(url, now.clone());
                            }
                            tracing::info!(
                                "Migrated old state format: {} processed URLs",
                                state.urls.len()
                            );
                            state
                        }
                        Err(_) => {
                            tracing::warn!(
                                "Failed to parse state file, this may be an incompatible format, starting fresh"
                            );
                            ProcessedState::new()
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::info!("No state file found: {}, starting fresh", e);
            ProcessedState::new()
        }
    }
}

pub fn save_state(state: &ProcessedState, path: &Path) -> Result<(), std::io::Error> {
    let contents = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, contents)?;
    tracing::info!("Saved state with {} processed URLs", state.urls.len());
    Ok(())
}
