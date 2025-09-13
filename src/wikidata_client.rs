use crate::data_structures::{WikidataEntity, WikidataFact};
use crate::cache::{save_entity_to_cache, load_entity_from_cache};
use reqwest::Client;
use serde_json::Value; // Keep serde_json::Value for manual parsing

pub async fn fetch_wikidata_entity(client: &Client, wikipedia_title: Option<&str>, wikidata_id: Option<&str>) -> Result<Option<WikidataEntity>, Box<dyn std::error::Error>> {
    let mut params = vec![
        ("action", "wbgetentities"),
        ("format", "json"),
        ("props", "labels|claims"),
    ];

    if let Some(title) = wikipedia_title {
        params.push(("sites", "enwiki"));
        params.push(("titles", title));
    } else if let Some(id) = wikidata_id {
        params.push(("ids", id));
    } else {
        return Ok(None);
    }

    let url = "https://www.wikidata.org/w/api.php";
    let res = client.get(url).query(&params).send().await?.json::<Value>().await?;

    let entities = res["entities"].as_object().ok_or("No entities found")?;
    if entities.is_empty() {
        return Ok(None);
    }

    // Iterate through entities to find the one with a valid ID and label
    let mut found_entity: Option<(String, Value)> = None;
    for (id, data) in entities {
        if id.starts_with("Q") { // Wikidata entity IDs start with 'Q'
            found_entity = Some((id.clone(), data.clone()));
            break;
        }
    }

    let (entity_id, entity_data) = found_entity.ok_or("No valid entity found")?;

    let label = entity_data["labels"]["en"]["value"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let mut facts = Vec::new();
    if let Some(claims) = entity_data["claims"].as_object() {
        for (property, property_claims) in claims {
            if let Some(claim_array) = property_claims.as_array() {
                for claim in claim_array {
                    if let Some(main_snak) = claim.get("mainsnak").and_then(Value::as_object) {
                        if let Some(data_value) = main_snak.get("datavalue").and_then(Value::as_object) {
                            if let Some(value_type) = data_value.get("type").and_then(Value::as_str) {
                                match value_type {
                                    "string" => {
                                        if let Some(value_str) = data_value.get("value").and_then(Value::as_str) {
                                            facts.push(WikidataFact { property: property.clone(), value: value_str.to_string() });
                                        }
                                    },
                                    "wikibase-entityid" => {
                                        if let Some(id_obj) = data_value.get("value").and_then(Value::as_object) {
                                            if let Some(entity_id_str) = id_obj.get("id").and_then(Value::as_str) {
                                                facts.push(WikidataFact { property: property.clone(), value: entity_id_str.to_string() });
                                            }
                                        }
                                    },
                                    "monolingualtext" => {
                                        if let Some(text_obj) = data_value.get("value").and_then(Value::as_object) {
                                            if let Some(text_str) = text_obj.get("text").and_then(Value::as_str) {
                                                facts.push(WikidataFact { property: property.clone(), value: text_str.to_string() });
                                            }
                                        }
                                    },
                                    "time" => {
                                        if let Some(time_obj) = data_value.get("value").and_then(Value::as_object) {
                                            if let Some(time_str) = time_obj.get("time").and_then(Value::as_str) {
                                                facts.push(WikidataFact { property: property.clone(), value: time_str.to_string() });
                                            }
                                        }
                                    },
                                    "quantity" => {
                                        if let Some(amount_str) = data_value.get("value").and_then(Value::as_object).and_then(|obj| obj.get("amount")).and_then(Value::as_str) {
                                            facts.push(WikidataFact { property: property.clone(), value: amount_str.to_string() });
                                        }
                                    },
                                    _ => {},
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Some(WikidataEntity {
        id: entity_id.clone(),
        label,
        facts,
    }))
}

pub async fn fetch_and_cache_wikidata_entity(client: &Client, wikipedia_title: Option<&str>, wikidata_id: Option<&str>) -> Result<Option<WikidataEntity>, Box<dyn std::error::Error>> {
    let query_id = if let Some(id) = wikidata_id {
        id.to_string()
    } else if let Some(title) = wikipedia_title {
        // A simple way to generate a consistent ID for caching based on title
        // In a real scenario, you might want to fetch the ID first and then use it
        title.replace(" ", "_").to_string()
    } else {
        return Ok(None);
    };

    // Try to load from cache first
    if let Ok(Some(entity)) = load_entity_from_cache(&query_id) {
        println!("Loaded Wikidata entity from cache: {}", query_id);
        return Ok(Some(entity));
    }

    println!("Fetching Wikidata entity from web: {}", query_id);
    let entity = fetch_wikidata_entity(client, wikipedia_title, wikidata_id).await?;

    if let Some(e) = &entity {
        // Use the actual Wikidata ID for caching if available
        save_entity_to_cache(e)?;
        println!("Saved Wikidata entity to cache: {}", e.id);
    }

    Ok(entity)
}