use crate::feed::FeedItem;

#[derive(Debug, Clone)]
pub struct ArticleText {
    pub title: String,
    pub body: String,
}

/// Extract article text from the RSS feed item.
/// Since article pages are behind Sucuri CloudProxy (JavaScript challenge),
/// we use the RSS description which contains a substantial summary (~55 words).
pub fn extract_article(item: &FeedItem) -> Result<ArticleText, Box<dyn std::error::Error>> {
    tracing::info!("Extracting article: {}", item.title);

    // Clean the RSS description: strip HTML tags and decode HTML entities
    let body = clean_description(&item.description);

    if body.trim().is_empty() {
        return Err(format!("Empty description for article '{}'", item.title).into());
    }

    // Also sanitize the title to remove Unicode characters that break TTS engines
    let title = sanitize_unicode(item.title.as_str());

    Ok(ArticleText {
        title,
        body,
    })
}

/// Normalize Unicode characters that may cause issues with TTS engines
/// (e.g., kokoro-micro panics on multi-byte chars at byte-boundary slicing).
fn sanitize_unicode(text: &str) -> String {
    text.replace("\u{2019}", "'")  // right single quotation mark → '
        .replace("\u{2018}", "'")  // left single quotation mark → '
        .replace("\u{201D}", "\"") // right double quotation mark → "
        .replace("\u{201C}", "\"") // left double quotation mark → "
        .replace("\u{2026}", "...") // horizontal ellipsis
        .replace("\u{00A0}", " ")  // non-breaking space
        .replace("\u{2003}", " ")  // em space
        .replace("\u{2002}", " ")  // en space
        .replace("\u{2014}", " — ") // em dash
        .replace("\u{2013}", " – ") // en dash
        .replace("\u{2011}", "-")   // non-breaking hyphen
        .replace("\u{2010}", "-")   // hyphen
        .replace("\u{200B}", "")    // zero-width space
        // Additional characters that may cause byte-boundary issues
        .replace("\u{2012}", "-")   // figure dash
        .replace("\u{2015}", " | ") // horizontal bar
        .replace("\u{203E}", "‾")   // overline
        .replace("\u{00B7}", "'")   // middle dot (used as apostrophe in some languages)
        .replace("\u{02BC}", "'")   // modifier letter apostrophe
        .replace("\u{FF07}", "'")   // fullwidth apostrophe
        .replace("\u{201B}", "'")   // single high-reversed-9 quotation mark
        .replace("\u{201A}", "'")   // single low-9 quotation mark
        .replace("\u{0060}", "'")   // grave accent → '
        .replace("\u{00AF}", "'")   // macron → '
}

/// Clean RSS description by stripping HTML tags and decoding HTML entities.
fn clean_description(desc: &str) -> String {
    // Remove HTML tags
    let no_tags = strip_html_tags(desc);
    // Decode common HTML entities
    let decoded = decode_html_entities(&no_tags);
    // Normalize Unicode characters
    let ascii_normalized = sanitize_unicode(&decoded);
    // Normalize whitespace (collapse multiple spaces)
    let words: Vec<&str> = ascii_normalized.split_whitespace().collect();
    words.join(" ")
}

fn strip_html_tags(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

fn decode_html_entities(text: &str) -> String {
    let mut decoded = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#8217;", "'")  // right single quote
        .replace("&#8216;", "'")  // left single quote
        .replace("&#8217;", "'")  // right single quote (duplicate codepoint)
        .replace("&#8216;", "'")  // left single quote (duplicate codepoint)
        .replace("&#8217;", "'")  // right single quote
        .replace("&#8216;", "'")  // left single quote
        .replace("&#8217;", "'")  // right single quote
        .replace("&#8216;", "'")  // left single quote
        .replace("&#8217;", "'")  // right single quote
        .replace("&#8216;", "'")  // left single quote
        .replace("&#8211;", " - ") // en dash
        .replace("&#8212;", " — ") // em dash
        .replace("&#8230;", "...") // ellipsis
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#39;", "'");
    // Clean up extra spaces around dashes
    while decoded.contains("  - ") {
        decoded = decoded.replace("  - ", " - ");
    }
    decoded
}
