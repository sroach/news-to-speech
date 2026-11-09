use chrono::{DateTime, Local};
use std::fs;
use std::path::Path;

pub fn generate_output_path(output_dir: &Path, article_index: usize) -> String {
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M");
    let filename = format!("{}_article-{}.wav", timestamp, article_index + 1);
    format!("{}/{}", output_dir.display(), filename)
}

pub fn ensure_output_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path)?;
    tracing::info!("Output directory ready: {:?}", path);
    Ok(())
}

/// Clean up WAV files older than the retention period.
/// Returns the number of deleted files.
pub fn cleanup_old_files(
    output_dir: &Path,
    retention_days: u64,
) -> Result<usize, Box<dyn std::error::Error>> {
    let cutoff = Local::now() - chrono::Duration::days(retention_days as i64);
    let mut deleted = 0;

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .wav files (skip processed.json and other files)
        if path.extension().and_then(|s| s.to_str()) != Some("wav") {
            continue;
        }

        let metadata = fs::metadata(&path)?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| DateTime::<Local>::from(t).into());

        if let Some(modified) = modified {
            if modified < cutoff {
                fs::remove_file(&path)?;
                tracing::debug!("Deleted old WAV file: {:?}", path.file_name());
                deleted += 1;
            }
        }
    }

    if deleted > 0 {
        tracing::info!(
            "Cleaned up {} WAV files older than {} days",
            deleted,
            retention_days
        );
    }

    Ok(deleted)
}
