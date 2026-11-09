mod article;
mod config;
mod feed;
mod output;
mod state;
mod tts;

use chrono::{DateTime, Local, Timelike};

use crate::config::Config;
use crate::state::ProcessedState;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            return;
        }
    };

    tracing::info!("Configuration loaded:");
    tracing::info!("  Feed URL: {}", config.feed_url);
    tracing::info!("  Output dir: {:?}", config.output_dir);
    tracing::info!("  Voice: {}", config.voice);
    tracing::info!("  Speed: {} wpm", config.speed);
    tracing::info!("  Interval: {}s", config.interval_secs);

    // Ensure output directory exists
    if let Err(e) = output::ensure_output_dir(&config.output_dir) {
        tracing::error!("Failed to create output directory: {}", e);
        return;
    }

    // Load processed state
    let state_path = config.output_dir.join("processed.json");
    let mut processed_state = state::load_state(&state_path);
    tracing::info!("Loaded {} processed URLs", processed_state.urls.len());

    // Run the pipeline immediately on first start
    tracing::info!("=== Starting initial pipeline ===");
    run_pipeline(&config, &mut processed_state, &state_path).await;

    // Main scheduler loop
    loop {
        // Wait until the next hour mark
        let now = Local::now();
        let next_hour = next_hour_mark(now);
        let sleep_duration = next_hour.signed_duration_since(now).to_std().unwrap();

        tracing::info!(
            "Next run at {}. Sleeping for {:?}...",
            next_hour.format("%Y-%m-%d %H:%M:%S"),
            sleep_duration
        );

        tokio::time::sleep(sleep_duration).await;

        // Run the pipeline
        tracing::info!("=== Starting hourly pipeline ===");
        run_pipeline(&config, &mut processed_state, &state_path).await;
    }
}

/// Calculate the next hour mark (minute 0, second 0) after the given time.
fn next_hour_mark(now: DateTime<Local>) -> DateTime<Local> {
    let current_hour = now.hour();
    if current_hour < 23 {
        now.with_hour(current_hour + 1)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    } else {
        // Midnight: roll over to next day at 01:00
        now.with_hour(1)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            + chrono::Days::new(1)
    }
}

async fn run_pipeline(
    config: &Config,
    processed_state: &mut ProcessedState,
    state_path: &std::path::PathBuf,
) {
    // Step 1: Fetch feed
    let items = match feed::fetch_feed(&config.feed_url).await {
        Ok(items) => items,
        Err(e) => {
            tracing::error!("Failed to fetch feed: {}", e);
            return;
        }
    };

    if items.is_empty() {
        tracing::info!("No items in feed");
        return;
    }

    tracing::info!("Fetched {} feed items", items.len());

    // Step 2: Filter out already-processed items
    let new_items: Vec<_> = items
        .iter()
        .filter(|item| {
            let url = if item.link.is_empty() {
                &item.title
            } else {
                &item.link
            };
            if processed_state.is_processed(url) {
                tracing::debug!("Skipping already processed: {}", url);
                false
            } else {
                true
            }
        })
        .collect();

    if new_items.is_empty() {
        tracing::info!("No new articles to process");
        return;
    }

    tracing::info!("{} new articles to process", new_items.len());

    // Step 3: Process each new article
    let mut success_count = 0;
    for (index, item) in new_items.iter().enumerate() {
        let url = if item.link.is_empty() {
            item.title.clone()
        } else {
            item.link.clone()
        };

        tracing::info!(
            "Processing article {}/{}: {}",
            index + 1,
            new_items.len(),
            item.title
        );

        // Extract article content from RSS description
        let article = match article::extract_article(item) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("Failed to extract article '{}': {}", item.title, e);
                continue;
            }
        };

        // Build the full text: title + body
        let full_text = format!("{}. {}", article.title, article.body);

        // Generate output path
        let output_path_str = output::generate_output_path(&config.output_dir, index);

        // Synthesize speech directly to WAV file using macOS `say`
        if let Err(e) = tts::synthesize_to_file(&full_text, &output_path_str, config) {
            tracing::error!("Failed to synthesize '{}': {}", item.title, e);
            continue;
        }

        // Mark as processed
        processed_state.mark_processed(url);
        success_count += 1;
    }

    // Save state
    if let Err(e) = state::save_state(processed_state, state_path) {
        tracing::error!("Failed to save state: {}", e);
    }

    tracing::info!(
        "Pipeline complete: {} articles processed successfully",
        success_count
    );
}
