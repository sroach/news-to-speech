use std::env;
use std::process::Command;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load config from config.toml (same as the main binary)
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    // Get text from CLI args or use a default sample
    let args: Vec<String> = env::args().skip(1).collect();
    let text = if args.is_empty() {
        "Good evening. This is a test of the news to speech system. \
         The quick brown fox jumps over the lazy dog. \
         Would you like to hear the latest news from Guyana?".to_string()
    } else {
        args.join(" ")
    };

    let output_path = "output/test_voice.wav";

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Synthesize speech using macOS `say` command
    match synthesize_to_file(&text, output_path, &config) {
        Ok(()) => {
            println!("Done! WAV written to {}", output_path);
        }
        Err(e) => {
            eprintln!("Synthesis failed: {}", e);
            std::process::exit(1);
        }
    }
}

// --- Config (mirrors src/config.rs) ---

struct Config {
    voice: String,
    speed: u32,
}

impl Config {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let builder = config::Config::builder()
            .set_default("voice", "Daniel")?
            .set_default("speed", 175u32)?;

        // Try to load config.toml from the current directory or parent
        let settings = builder
            .add_source(config::File::with_name("config").required(false))
            .build()?;

        let voice: String = settings.get("voice")?;
        let speed: u32 = settings.get("speed")?;

        Ok(Config { voice, speed })
    }
}

// --- TTS ---

fn synthesize_to_file(
    text: &str,
    output_path: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Synthesizing with voice '{}', speed {}", config.voice, config.speed);

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
        .output()?
    ;

    let _ = std::fs::remove_file(&text_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("'say' command failed: {}", stderr).into());
    }

    tracing::info!("Wrote WAV file: {}", output_path);
    Ok(())
}
