use chrono::Local;
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
