# News to Speech

A Rust daemon that fetches the Guyana Chronicle RSS feed, converts each article to speech, and drops WAV files into a directory that Liquidsoap monitors for its playlist.

Supports two TTS engines: **Kokoro TTS** (default, natural-sounding AI voices) and macOS's built-in `say` command (backup).

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
| `tts_engine` | `kokoro` | TTS engine: `"kokoro"` or `"say"` |
| `voice` | `af_heart` | Voice name (see Available Voices below) |
| `speed` | `175` | Speech rate in wpm (only used with `say` engine) |
| `interval_secs` | `3600` | Seconds between fetch cycles |
| `retention_days` | `5` | Days to keep WAV files (older files are auto-deleted) |

### Available Voices

#### Kokoro TTS Voices (default engine)

Kokoro provides 54 natural-sounding voices across 8 languages. On first run, the model (~310MB) is automatically downloaded to `~/.cache/k/`.

**American English — 20 voices**

| Voice | Gender | Voice | Gender |
|-------|--------|-------|--------|
| `af_heart` | Female | `am_adam` | Male |
| `af_alloy` | Female | `am_echo` | Male |
| `af_aoede` | Female | `am_eric` | Male |
| `af_bella` | Female | `am_fenrir` | Male |
| `af_jessica` | Female | `am_liam` | Male |
| `af_kore` | Female | `am_michael` | Male |
| `af_nicole` | Female | `am_onyx` | Male |
| `af_nova` | Female | `am_puck` | Male |
| `af_river` | Female | `am_santa` | Male |
| `af_sarah` | Female | | |
| `af_sky` | Female | | |

**British English — 8 voices**

| Voice | Gender | Voice | Gender |
|-------|--------|-------|--------|
| `bf_alice` | Female | `bm_daniel` | Male |
| `bf_emma` | Female | `bm_fable` | Male |
| `bf_isabella` | Female | `bm_george` | Male |
| `bf_lily` | Female | `bm_lewis` | Male |

**Other Languages — 26 voices**

| Language | Voices |
|----------|--------|
| Spanish | `ef_alpha` (F), `em_alpha` (M) |
| French | `ff_alpha` (F), `fm_alpha` (M) |
| German | `pf_alpha` (F), `pm_alpha` (M) |
| Italian | `if_sara` (F), `im_nicola` (M) |
| Portuguese (BR) | `pf_dora` (F), `pm_alex` (M), `pm_santa` (M) |
| Hindi | `hf_alpha` (F), `hf_beta` (F), `hm_omega` (M), `hm_psi` (M) |
| Chinese (Mandarin) | 8 voices (`zf_*` and `zm_*` variants) |
| Japanese | 5 voices (`jf_*` and `jm_*` variants) |

#### macOS Voices (backup engine)

When `tts_engine = "say"`, use macOS built-in voices:

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

List all available macOS voices:

```bash
say -v '?'
```

#### Switching Engines

To switch from Kokoro to macOS `say` (or vice versa), update `config.toml`:

```toml
# Use Kokoro TTS (default)
tts_engine = "kokoro"
voice = "af_heart"

# Or use macOS say (backup)
# tts_engine = "say"
# voice = "Samantha"
```

### Environment Variables

Override any config setting with `NEWS_` prefixed environment variables:

```bash
# Switch to macOS say engine
NEWS_TTS_ENGINE=say NEWS_VOICE=Samantha ./target/release/newstospeech

# Use a different Kokoro voice
NEWS_VOICE=am_michael ./target/release/newstospeech
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
├── index.html          ← Web player (open in browser)
├── manifest.json       ← Auto-updated file list
├── processed.json      ← Deduplication state
├── 2025-11-08_20-00_article-1.wav
├── 2025-11-08_20-00_article-2.wav
└── 2025-11-08_20-00_article-3.wav
```

Each file is 16-bit mono PCM at 24kHz (Kokoro) or 22050 Hz (macOS `say`) — compatible with Liquidsoap.

### Web Player

On startup, the app writes `index.html` and `manifest.json` to the output directory. Point your browser at the output directory (served by nginx or any static file server) to access the audio player:

- **Auto-refresh** — The player polls `manifest.json` every 30 seconds and shows new files with a "NEW" badge.
- **Click to play** — Click any file in the list to play it in the browser's built-in audio player.
- **No download needed** — WAV files stream directly in the browser.

#### Serving with Nginx

Add a location block to serve the output directory:

```nginx
location /news/ {
    alias /path/to/output/;
    autoindex off;
    default_type application/json;

    # Serve index.html as the default
    try_files $uri $uri/ /index.html;

    # Cache WAV files; no cache for manifest
    location ~* \.wav$ {
        expires 1d;
    }
    location = /news/manifest.json {
        add_header Cache-Control "no-cache";
    }
}
```

Then open `http://your-server/news/` in your browser.

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

- **macOS** — Primary platform; Kokoro TTS also supports Linux and Windows.
- **Rust** — Build requires the Rust toolchain (`rustup.rs`).
- **Kokoro TTS model** — On first run with `tts_engine = "kokoro"`, the ONNX model (~310MB) is automatically downloaded to `~/.cache/k/`. Subsequent runs use the cached model.
- **macOS `say` backup** — When using `tts_engine = "say"`, only macOS is supported.

## Reset Processed Articles

To re-process all articles (e.g., after changing the voice), delete the state file:

```bash
rm output/processed.json
```

The next run will treat all articles as new. Alternatively, old entries are automatically pruned after `retention_days` (default 5 days), so articles that reappear in the feed after that window will be re-processed.
