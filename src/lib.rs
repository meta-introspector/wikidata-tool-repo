pub mod data_structures;
pub mod wikipedia_parser;
pub mod wikidata_client;
pub mod cache;

pub use data_structures::{WikipediaArticle, WikidataFact, WikidataEntity};
pub use wikipedia_parser::extract_article_data;
pub use wikidata_client::fetch_wikidata_entity;
pub use cache::{save_article_to_cache, load_article_from_cache, save_entity_to_cache, load_entity_from_cache};