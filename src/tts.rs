use crate::config::Config;
use std::process::Command;

/// Synthesize text to speech using macOS `say` command.
/// Returns the path to the generated WAV file.
pub fn synthesize_to_file(
    text: &str,
    output_path: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Synthesizing with voice '{}', speed {} wpm, output: {}",
        config.voice,
        config.speed,
        output_path
    );

    // Escape text for shell safety by using --file=- and writing via stdin
    // But say doesn't support stdin well, so we use a temp file approach
    // Actually, say supports direct string args; we need to be careful with shell escaping
    // The safest approach: write text to a temp file and use -f flag
    let text_file = format!("{}.tmp", output_path);
    std::fs::write(&text_file, text)?;

    let output = Command::new("say")
        .arg("-v")
        .arg(&config.voice)
        .arg("-r")
        .arg(config.speed.to_string())
        .arg("-o")
        .arg(output_path)
        .arg("--data-format=LEI16@22050")
        .arg("-f")
        .arg(&text_file)
        .output()
        .map_err(|e| {
            tracing::error!("Failed to execute 'say' command: {}", e);
            format!("Failed to execute 'say': {}", e)
        })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&text_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("'say' command failed: {}", stderr);
        return Err(format!("'say' command failed: {}", stderr).into());
    }

    tracing::info!("Wrote WAV file: {}", output_path);
    Ok(())
}
