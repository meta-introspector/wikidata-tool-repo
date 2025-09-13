use std::path::{Path, PathBuf};
use std::fs;

use crate::data_structures::{WikipediaArticle, WikidataEntity};

const WIKIPEDIA_CACHE_DIR: &str = "wikipedia_extractor/cache/wikipedia";
const WIKIDATA_CACHE_DIR: &str = "wikipedia_extractor/cache/wikidata";

fn get_wikipedia_cache_path(title: &str) -> PathBuf {
    let filename = format!("{}.json", sanitize_filename(title));
    Path::new(WIKIPEDIA_CACHE_DIR).join(filename)
}

fn get_wikidata_cache_path(id: &str) -> PathBuf {
    let filename = format!("{}.json", sanitize_filename(id));
    Path::new(WIKIDATA_CACHE_DIR).join(filename)
}

fn sanitize_filename(name: &str) -> String {
    name.replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace('*', "_")
        .replace('?', "_")
        .replace('"', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace('|', "_")
}

pub fn save_article_to_cache(article: &WikipediaArticle) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_wikipedia_cache_path(&article.title);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?; // Create parent directories if they don't exist
    }
    let json = serde_json::to_string_pretty(article)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn load_article_from_cache(title: &str) -> Result<Option<WikipediaArticle>, Box<dyn std::error::Error>> {
    let path = get_wikipedia_cache_path(title);
    match fs::read_to_string(&path) {
        Ok(json) => {
            let article: WikipediaArticle = serde_json::from_str(&json)?;
            Ok(Some(article))
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn save_entity_to_cache(entity: &WikidataEntity) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_wikidata_cache_path(&entity.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?; // Create parent directories if they don't exist
    }
    let json = serde_json::to_string_pretty(entity)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn load_entity_from_cache(id: &str) -> Result<Option<WikidataEntity>, Box<dyn std::error::Error>> {
    let path = get_wikidata_cache_path(id);
    match fs::read_to_string(&path) {
        Ok(json) => {
            let entity: WikidataEntity = serde_json::from_str(&json)?;
            Ok(Some(entity))
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
