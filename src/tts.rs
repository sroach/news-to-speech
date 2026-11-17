use crate::config::Config;
use std::process::Command;

/// Synthesize text to speech and write to a WAV file.
/// Uses Kokoro TTS or macOS `say` depending on `config.tts_engine`.
pub async fn synthesize_to_file(
    text: &str,
    output_path: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.tts_engine.as_str() {
        "kokoro" => synthesize_kokoro(text, output_path, config).await,
        "say" => synthesize_say(text, output_path, config),
        other => {
            return Err(format!("Unknown TTS engine: '{}'. Use 'kokoro' or 'say'.", other).into());
        }
    }
}

/// Normalize Unicode characters that may cause issues with Kokoro TTS
/// (e.g., panics on multi-byte chars at byte-boundary slicing in G2P).
fn sanitize_for_kokoro(text: &str) -> String {
    text.replace("\u{2019}", "'")  // right single quotation mark
        .replace("\u{2018}", "'")  // left single quotation mark
        .replace("\u{201D}", "\"") // right double quotation mark
        .replace("\u{201C}", "\"") // left double quotation mark
        .replace("\u{2026}", "...") // horizontal ellipsis
        .replace("\u{00A0}", " ")  // non-breaking space
        .replace("\u{2003}", " ")  // em space
        .replace("\u{2002}", " ")  // en space
        .replace("\u{2014}", " - ") // em dash
        .replace("\u{2013}", " - ") // en dash
        .replace("\u{2012}", "-")   // figure dash
        .replace("\u{2011}", "-")   // non-breaking hyphen
        .replace("\u{2010}", "-")   // hyphen
        .replace("\u{200B}", "")    // zero-width space
        .replace("\u{00B7}", "'")   // middle dot
        .replace("\u{02BC}", "'")   // modifier letter apostrophe
        .replace("\u{FF07}", "'")   // fullwidth apostrophe
        .replace("\u{201B}", "'")   // single high-reversed-9 quotation mark
        .replace("\u{201A}", "'")   // single low-9 quotation mark
        .replace("\u{0060}", "'")   // grave accent
        .replace("\u{00AF}", "'")   // macron
}

/// Synthesize text to speech using Kokoro TTS.
async fn synthesize_kokoro(
    text: &str,
    output_path: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Synthesizing with Kokoro, voice '{}', output: {}",
        config.voice,
        output_path
    );

    // Sanitize text to prevent kokoro-micro G2P byte-boundary panics
    let sanitized_text = sanitize_for_kokoro(text);

    let engine = kokoro_micro::TtsEngine::new()
        .await
        .map_err(|e| format!("Failed to initialize Kokoro: {}", e))?;

    let audio = engine
        .synthesize_with_options(&sanitized_text, Some(&config.voice), 1.0, 1.0, None)
        .map_err(|e| format!("Kokoro synthesis failed: {}", e))?;

    engine
        .save_wav(output_path, &audio)
        .map_err(|e| format!("Failed to save WAV: {}", e))?;

    tracing::info!("Wrote WAV file: {}", output_path);
    Ok(())
}

/// Synthesize text to speech using macOS `say` command (backup engine).
fn synthesize_say(
    text: &str,
    output_path: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Synthesizing with macOS 'say', voice '{}', speed {} wpm, output: {}",
        config.voice,
        config.speed,
        output_path
    );

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

    let _ = std::fs::remove_file(&text_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("'say' command failed: {}", stderr);
        return Err(format!("'say' command failed: {}", stderr).into());
    }

    tracing::info!("Wrote WAV file: {}", output_path);
    Ok(())
}
