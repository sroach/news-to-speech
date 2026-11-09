use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedState {
    pub urls: HashSet<String>,
}

impl ProcessedState {
    pub fn new() -> Self {
        ProcessedState {
            urls: HashSet::new(),
        }
    }

    pub fn is_processed(&self, url: &str) -> bool {
        self.urls.contains(url)
    }

    pub fn mark_processed(&mut self, url: String) {
        self.urls.insert(url);
    }
}

pub fn load_state(path: &Path) -> ProcessedState {
    match fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<ProcessedState>(&contents) {
                Ok(state) => {
                    tracing::info!("Loaded state with {} processed URLs", state.urls.len());
                    state
                }
                Err(e) => {
                    tracing::warn!("Failed to parse state file: {}, starting fresh", e);
                    ProcessedState::new()
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
