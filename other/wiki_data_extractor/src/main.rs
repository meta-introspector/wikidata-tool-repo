use wikipedia_extractor::data_structures::{WikipediaArticle, WikidataEntity};
use wikipedia_extractor::wikipedia_parser::extract_article_data;
use wikipedia_extractor::wikidata_client::fetch_and_cache_wikidata_entity;
use reqwest::Client;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("MyRustWikipediaExtractor/1.0 (contact@example.com)")
        .build()?;

    let initial_wikipedia_url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    println!("\n--- Processing Wikipedia Article: {} ---", initial_wikipedia_url);

    // Fetch and extract Wikipedia article
    let res = client.get(initial_wikipedia_url).send().await?;
    let body = res.text().await?;
    let article = extract_article_data(&body, initial_wikipedia_url)
        .ok_or("Failed to extract article data")?;

    println!("Extracted Title: {}", article.title);
    println!("Extracted {} links.", article.links.len());

    let base_url = Url::parse("https://en.wikipedia.org").unwrap();

    // Process links and fetch Wikidata entities for Wikipedia links
    for link in article.links {
        println!("  Processing link: {}", link.href);
        if let Ok(url) = base_url.join(&link.href) {
            if url.domain() == Some("en.wikipedia.org") && url.path().starts_with("/wiki/") && !url.path().contains(":") && !url.path().contains("#") {
                if url.domain() == Some("en.wikipedia.org") && url.path().starts_with("/wiki/") && !url.path().contains(":") && !url.path().contains("#") &&
               !url.path().starts_with("/wiki/Special:") &&
               !url.path().starts_with("/wiki/Wikipedia:") &&
               !url.path().starts_with("/wiki/File:") &&
               !url.path().starts_with("/wiki/Category:") &&
               !url.path().starts_with("/wiki/Template:") &&
               !url.path().starts_with("/wiki/Help:") &&
               !url.path().starts_with("/wiki/Portal:") &&
               !url.path().starts_with("/wiki/Talk:") {
                let title = url.path().trim_start_matches("/wiki/").replace(" ", "_");
                println!("\n  --- Attempting to fetch Wikidata for Wikipedia Link: {} ---", title);
                let entity_result = fetch_and_cache_wikidata_entity(&client, Some(&title), None).await;
                match entity_result {
                    Ok(Some(entity)) => {
                        println!("    Wikidata Entity ID: {}", entity.id);
                        println!("    Wikidata Entity Label: {}", entity.label);
                        println!("    Wikidata Facts (Property: Value):");
                        for fact in entity.facts {
                            println!("      - {}: {}", fact.property, fact.value);
                        }
                    },
                    Ok(None) => {
                        println!("    No Wikidata entity found for {}", title);
                    },
                    Err(e) => {
                        eprintln!("    Error fetching Wikidata for {}: {}", title, e);
                    }
                }
            }
        }
    }

    Ok(())
}
}