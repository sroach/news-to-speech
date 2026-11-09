use feed_rs::parser;
use reqwest;

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: Option<String>,
}

pub async fn fetch_feed(url: &str) -> Result<Vec<FeedItem>, Box<dyn std::error::Error>> {
    tracing::info!("Fetching feed from: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()?;
    let response = client.get(url).send().await?.error_for_status()?;

    let text = response.text().await?;
    tracing::debug!("Raw feed content length: {} bytes", text.len());

    let feed = parser::parse(text.as_bytes())?;

    let items: Vec<FeedItem> = feed
        .entries
        .iter()
        .map(|entry| FeedItem {
            title: entry
                .title
                .as_ref()
                .map(|t| t.content.clone())
                .unwrap_or_default(),
            link: entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default(),
            description: entry
                .summary
                .as_ref()
                .map(|s| s.content.clone())
                .unwrap_or_default(),
            pub_date: entry.published.map(|dt| dt.to_string()),
        })
        .collect();

    tracing::info!("Parsed {} items from feed", items.len());
    Ok(items)
}
