# News to Speech

A Rust daemon that fetches the Guyana Chronicle RSS feed, converts each article to speech using Apple's built-in text-to-speech, and drops WAV files into a directory that Liquidsoap monitors for its playlist.

Designed for Apple Silicon (macOS) — uses the native `say` command for high-quality neural voices.

## Quick Start

```bash
# Build
cargo build --release

# Run (fetches news immediately, then runs on the hour)
./target/release/newstospeech
```

WAV files will appear in `./output/`.

## Configuration

Edit `config.toml` in the project root:

| Setting | Default | Description |
|---------|---------|-------------|
| `feed_url` | `https://guyanachronicle.com/feed/` | RSS feed URL |
| `output_dir` | `./output` | Where WAV files are written |
| `voice` | `Daniel` | macOS voice name |
| `speed` | `175` | Speech rate (words per minute) |
| `interval_secs` | `3600` | Seconds between fetch cycles |
| `retention_days` | `5` | Days to keep WAV files (older files are auto-deleted) |

### Available Voices

The app uses macOS's built-in `say` command. High-quality neural voices for news reading:

| Voice | Accent | Gender |
|-------|--------|--------|
| **Samantha** | US English | Female |
| **Daniel** | UK English | Male |
| **Reed** | US/UK English | Male |
| **Shelley** | US/UK English | Female |
| **Rocko** | US/UK English | Male |
| **Sandy** | US/UK English | Female |
| **Eddy** | US/UK English | Male |
| **Flo** | US/UK English | Female |

List all available voices:

```bash
say -v '?'
```

### Environment Variables

Override any config setting with `NEWS_` prefixed environment variables:

```bash
NEWS_VOICE=Samantha NEWS_SPEED=150 ./target/release/newstospeech
```

## Testing Voice Output

Preview the voice without waiting for the scheduler:

```bash
# Default sample text
cargo run --bin test_voice

# Custom text
cargo run --bin test_voice "Good evening, this is your Guyana Chronicle news update."
```

This writes `output/test_voice.wav` using the voice and speed from `config.toml`.

## How It Works

1. **On startup** — The app immediately fetches the RSS feed and processes any new articles.
2. **On the hour** — The app sleeps until the next hour mark (e.g., 21:00, 22:00), then repeats the pipeline.
3. **Deduplication** — Processed article URLs are saved to `output/processed.json` with timestamps. Re-running the app will not regenerate already-processed articles.
4. **Article content** — Each RSS item's description is cleaned (HTML tags stripped, entities decoded) and used as the spoken text. Full article pages are behind a JavaScript challenge, so the RSS description (~55 words) is used instead.
5. **Auto cleanup** — After each pipeline run, WAV files and processed state entries older than `retention_days` (default 5 days) are automatically deleted. This keeps the output directory from growing indefinitely and allows old articles to be re-processed if they reappear in the feed after the retention window.

### Output Files

WAV files are written with timestamped filenames:

```
output/
├── 2025-11-08_20-00_article-1.wav
├── 2025-11-08_20-00_article-2.wav
├── 2025-11-08_20-00_article-3.wav
└── processed.json
```

Each file is 16-bit mono PCM at 22050 Hz — compatible with Liquidsoap.

## Liquidsoap Integration

Liquidsoap can watch the output directory for new WAV files. Example configuration:

```ocaml
# Watch the output directory for new WAV files
def news_playlist() =
  playlist(reload_mode="watch", "~/.newstospeech/output/")
end

# Rotate through news items
rotate([
  track(~weight=1, news_playlist()),
  track(~weight=9, main_music())
])
```

Or simply point Liquidsoap's playlist at the output directory with `reload_mode="watch"`.

## Logging

Log level is controlled by the `RUST_LOG` environment variable:

```bash
# Default: info
./target/release/newstospeech

# Debug level for troubleshooting
RUST_LOG=debug ./target/release/newstospeech

# Error only
RUST_LOG=error ./target/release/newstospeech
```

## Requirements

- **macOS** — The `say` command is macOS-specific.
- **Rust** — Build requires the Rust toolchain (`rustup.rs`).
- No external TTS dependencies — uses the built-in `say` command.

## Reset Processed Articles

To re-process all articles (e.g., after changing the voice), delete the state file:

```bash
rm output/processed.json
```

The next run will treat all articles as new. Alternatively, old entries are automatically pruned after `retention_days` (default 5 days), so articles that reappear in the feed after that window will be re-processed.
