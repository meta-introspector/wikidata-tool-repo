use wikipedia_extractor::data_structures::{WikipediaArticle, WikidataEntity, WikidataFact};
use wikipedia_extractor::wikipedia_parser::extract_article_data;
use wikipedia_extractor::wikidata_client::{fetch_wikidata_entity, fetch_and_cache_wikidata_entity}; // Import fetch_and_cache_wikidata_entity

#[tokio::test] async fn test_fetch_and_extract_wikipedia_page() -> Result<(), Box<dyn std::error::Error>> { // Add Result return type
    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    let client = reqwest::Client::builder()
        .user_agent("MyRustWikipediaExtractor/1.0 (contact@example.com)")
        .build().unwrap();
    let res = client.get(url).send().await?; // Use ? operator
    let body = res.text().await?; // Use ? operator
    
    let article = extract_article_data(&body, url)
        .ok_or("Failed to extract article data from fetched content")?; // Handle Option
    
    println!("Extracted Title: {}", article.title);
    println!("Extracted Content (partial): {}...", &article.content[..std::cmp::min(article.content.len(), 500)]);
    println!("Extracted Links (first 5): {:?}\n", &article.links[..std::cmp::min(article.links.len(), 5)]);

    assert!(!article.title.is_empty(), "Extracted title is empty");
    assert!(article.content.contains("Rust"), "Content does not contain 'Rust'");
    assert!(article.content.contains("programming language"), "Content does not contain 'programming language'");
    assert!(!article.links.is_empty(), "No links extracted");
    assert!(article.links.iter().any(|link| link.contains("/wiki/Programming_language")), "Did not find expected link");
    Ok(())
}

#[tokio::test] async fn test_fetch_wikidata_entity_by_wikipedia_title() -> Result<(), Box<dyn std::error::Error>> { // Add Result return type
    let client = reqwest::Client::builder()
        .user_agent("MyRustWikipediaExtractor/1.0 (contact@example.com)")
        .build().unwrap();
    
    let entity = fetch_and_cache_wikidata_entity(&client, Some("Rust (programming language)"), None).await?; // Use ? operator
    assert!(entity.is_some(), "Failed to fetch Wikidata entity");
    
    let entity = entity.unwrap();
    println!("Wikidata Entity ID: {}", entity.id);
    println!("Wikidata Entity Label: {}", entity.label);
    println!("Wikidata Facts: {:?}\n", entity.facts);

    assert!(entity.id == "Q768046" || entity.id == "Q575650"); // Accept either ID for Rust
    assert!(entity.label.contains("Rust"));
    assert!(!entity.facts.is_empty(), "No facts extracted");
    assert!(entity.facts.iter().any(|fact| fact.property == "P31" && fact.value == "Q78383"), "Did not find expected instance of fact"); // P31: instance of, Q78383: programming language
    Ok(())
}

#[tokio::test] async fn test_fetch_wikidata_entity_by_wikidata_id() -> Result<(), Box<dyn std::error::Error>> { // Add Result return type
    let client = reqwest::Client::builder()
        .user_agent("MyRustWikipediaExtractor/1.0 (contact@example.com)")
        .build().unwrap();
    
    let entity = fetch_and_cache_wikidata_entity(&client, None, Some("Q768046")).await?; // Use ? operator
    assert!(entity.is_some(), "Failed to fetch Wikidata entity");
    
    let entity = entity.unwrap();
    println!("Wikidata Entity ID: {}", entity.id);
    println!("Wikidata Entity Label: {}", entity.label);
    println!("Wikidata Facts: {:?}\n", entity.facts);

    assert_eq!(entity.id, "Q768046"); // Wikidata ID for Rust (programming language)
    assert!(entity.label.contains("Rust"));
    assert!(!entity.facts.is_empty(), "No facts extracted");
    assert!(entity.facts.iter().any(|fact| fact.property == "P31" && fact.value == "Q78383"), "Did not find expected instance of fact"); // P31: instance of, Q78383: programming language
    Ok(())
}


