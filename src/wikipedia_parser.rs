use crate::data_structures::{WikipediaArticle, WikipediaLink};
use crate::cache::{save_article_to_cache, load_article_from_cache};
use wikipedia::{Wikipedia}; // Import Wikipedia from the wikipedia crate
use wikipedia::http::default::Client as WikipediaHttpClient;

pub fn extract_article_data(html_content: &str, url: &str) -> Option<WikipediaArticle> {
    let document = scraper::Html::parse_document(html_content);
    let title_selector = scraper::Selector::parse("h1#firstHeading").unwrap();
    let content_selector = scraper::Selector::parse("div#mw-content-text").unwrap();
    let link_selector = scraper::Selector::parse("a").unwrap();

    let title = document.select(&title_selector).next().map(|e| e.text().collect::<String>())?;
    let content = document.select(&content_selector).next().map(|e| e.text().collect::<String>())?;

    let links: Vec<WikipediaLink> = document.select(&link_selector)
        .filter_map(|element| {
            let href = element.value().attr("href")?.to_string();
            let text = element.text().collect::<String>();
            Some(WikipediaLink { href, text })
        })
        .collect();

    Some(WikipediaArticle {
        title,
        content,
        url: url.to_string(),
        links,
        revision_id: None, // We don't have revision ID from this method
    })
}

pub async fn fetch_and_cache_wikipedia_article(url: &str, title: &str) -> Result<WikipediaArticle, Box<dyn std::error::Error>> {
    // Try to load from cache first
    if let Ok(Some(article)) = load_article_from_cache(title) {
        println!("Loaded Wikipedia article from cache: {}", title);
        return Ok(article);
    }

    println!("Fetching Wikipedia article from web: {}", url);

    let wiki = Wikipedia::new(WikipediaHttpClient::default()); // Use default client
    let page = wiki.page_from_title(title.to_string()); // Convert &str to String

    let page_content = page.get_content()?;
    let page_url = url.to_string(); // Re-introducing this line

    let article_data = extract_article_data(&page_content, url)
        .ok_or("Failed to extract article data from fetched content")?;

    let article = WikipediaArticle {
        title: article_data.title,
        url: page_url,
        revision_id: None, // The wikipedia crate doesn't directly expose revision ID in this way
        content: article_data.content,
        links: article_data.links,
    };

    // Save to cache
    save_article_to_cache(&article)?;
    println!("Saved Wikipedia article to cache: {}", title);

    Ok(article)
}
