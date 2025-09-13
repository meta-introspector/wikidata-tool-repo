use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct WikipediaLink {
    pub href: String,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct WikipediaArticle {
    pub title: String,
    pub url: String,
    pub revision_id: Option<u64>,
    pub content: String,
    pub links: Vec<WikipediaLink>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct WikidataFact {
    pub property: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct WikidataEntity {
    pub id: String,
    pub label: String,
    pub facts: Vec<WikidataFact>,
}
