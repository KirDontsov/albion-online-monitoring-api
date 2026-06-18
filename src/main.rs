#![allow(dead_code)]
mod error;
mod items;
mod artefacts;
mod resources;
mod prices;
mod person;

use actix_cors::Cors;
use actix_web::web::{BytesMut, Json, Payload};
use actix_web::{get, http, post, App, HttpServer, Responder, Result};
use futures::StreamExt;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

#[derive(Serialize, Deserialize, Debug)]
struct Data {
	items: String,
	locations: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Serialize, Deserialize)]
struct RecipeResource {
	item_id: String,
	count: String,
}

#[derive(Deserialize)]
struct Recipe {
	item_id: String,
	artefact_id: String,
	resources: Vec<RecipeResource>,
}

fn tier_label(item_id: &str) -> String {
	match item_id.as_bytes()[1] {
		b'2' => "T2".to_string(),
		b'3' => "T3".to_string(),
		b'4' => "T4".to_string(),
		b'5' => "T5".to_string(),
		b'6' => "T6".to_string(),
		b'7' => "T7".to_string(),
		b'8' => "T8".to_string(),
		_ => "T?".to_string(),
	}
}

fn make_label(item_id: &str) -> String {
	let tier = tier_label(item_id);
	let name_part = &item_id[3..];
	let readable = name_part.replace('_', " ");
	format!("{} {}", tier, readable)
}

async fn seed_items_from_recipes() -> Result<(), Box<dyn std::error::Error>> {
	let data = fs::read_to_string("data/item_recipes.json")?;
	let recipes: Vec<Recipe> = serde_json::from_str(&data)?;

	let ru_data = fs::read_to_string("data/item_names_ru.json")?;
	let ru_names: HashMap<String, String> = serde_json::from_str(&ru_data)?;

	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.to_string();

	// Collect unique base items (no @) and unique artefact IDs
	let mut base_items: HashMap<String, &Recipe> = HashMap::new();
	let mut artefact_ids: HashMap<String, &Recipe> = HashMap::new();
	let mut resource_ids: HashMap<String, bool> = HashMap::new();

	for recipe in &recipes {
		let base_id = recipe.item_id.split('@').next().unwrap_or(&recipe.item_id);
		base_items.entry(base_id.to_string()).or_insert(recipe);
		artefact_ids.entry(recipe.artefact_id.clone()).or_insert(recipe);
		for res in &recipe.resources {
			resource_ids.insert(res.item_id.clone(), true);
		}
	}

	// Check existing records
	let existing_items: Vec<serde_json::Value> = DB.select("items").await?;
	let existing_item_ids: HashMap<String, bool> = existing_items
		.iter()
		.filter_map(|v| v.get("item_id").and_then(|id| id.as_str()))
		.map(|id| (id.to_string(), true))
		.collect();

	let existing_artefacts: Vec<serde_json::Value> = DB.select("artefacts").await?;
	let existing_artefact_ids: HashMap<String, bool> = existing_artefacts
		.iter()
		.filter_map(|v| v.get("item_id").and_then(|id| id.as_str()))
		.map(|id| (id.to_string(), true))
		.collect();

	let existing_resources: Vec<serde_json::Value> = DB.select("resources").await?;
	let existing_resource_ids: HashMap<String, bool> = existing_resources
		.iter()
		.filter_map(|v| v.get("item_id").and_then(|id| id.as_str()))
		.map(|id| (id.to_string(), true))
		.collect();

	let mut items_created = 0u32;
	let mut artefacts_created = 0u32;
	let mut resources_created = 0u32;

	// Seed items
	for (item_id, recipe) in &base_items {
		if existing_item_ids.contains_key(item_id) {
			continue;
		}
		let label = ru_names.get(item_id.as_str()).cloned().unwrap_or_else(|| make_label(item_id));
		let artefact_id = &recipe.artefact_id;
		let resources_str = serde_json::to_string(&recipe.resources).unwrap_or_else(|_| "[]".to_string());
		DB.query("CREATE items SET label=$label, item_id=$item_id, craft_price='0', enchantment_price='0', artefact_id=$artefact_id, sell_price_fort_sterling='0', sell_price_martlock='0', sell_price_thetford='0', sell_price_brecilien='0', buy_price_fort_sterling='0', buy_price_martlock='0', buy_price_thetford='0', buy_price_brecilien='0', orders_thetford='0', orders_fort_sterling='0', orders_martlock='0', orders_brecilien='0', created_at=$now, updated_at=$now, resources=$resources, comment='', popularity='0', source='api'")
			.bind(("label", label))
			.bind(("item_id", item_id.clone()))
			.bind(("artefact_id", artefact_id.clone()))
			.bind(("resources", resources_str))
			.bind(("now", now.clone()))
			.await?;
		items_created += 1;
	}

	// Seed artefacts
	for (artefact_id, recipe) in &artefact_ids {
		if existing_artefact_ids.contains_key(artefact_id) {
			continue;
		}
		let label = ru_names.get(artefact_id.as_str()).cloned().unwrap_or_else(|| make_label(artefact_id));
		let crafted_item_id = recipe.item_id.split('@').next().unwrap_or(&recipe.item_id);
		DB.query("CREATE artefacts SET label=$label, item_id=$item_id, crafted_item_id=$crafted_item_id, sell_price_fort_sterling='0', sell_price_martlock='0', sell_price_thetford='0', sell_price_brecilien='0', buy_price_fort_sterling='0', buy_price_martlock='0', buy_price_thetford='0', buy_price_brecilien='0', orders_thetford='0', orders_fort_sterling='0', orders_martlock='0', orders_brecilien='0', created_at=$now, updated_at=$now, comment='', popularity='0', source='api'")
			.bind(("label", label))
			.bind(("item_id", artefact_id.clone()))
			.bind(("crafted_item_id", crafted_item_id.to_string()))
			.bind(("now", now.clone()))
			.await?;
		artefacts_created += 1;
	}

	// Seed resources
	for resource_id in resource_ids.keys() {
		if existing_resource_ids.contains_key(resource_id) {
			continue;
		}
		let label = ru_names.get(resource_id.as_str()).cloned().unwrap_or_else(|| make_label(resource_id));
		DB.query("CREATE resources SET label=$label, item_id=$item_id, craft_price='0', sell_price_fort_sterling='0', sell_price_martlock='0', sell_price_thetford='0', sell_price_brecilien='0', buy_price_fort_sterling='0', buy_price_martlock='0', buy_price_thetford='0', buy_price_brecilien='0', orders_thetford='0', orders_fort_sterling='0', orders_martlock='0', orders_brecilien='0', created_at=$now, updated_at=$now, comment='', popularity='0', source='api'")
			.bind(("label", label))
			.bind(("item_id", resource_id.clone()))
			.bind(("now", now.clone()))
			.await?;
		resources_created += 1;
	}

	println!(
		"Seeded: {} items, {} artefacts, {} resources",
		items_created, artefacts_created, resources_created
	);
	Ok(())
}

async fn update_labels_with_ru_names() -> Result<(), Box<dyn std::error::Error>> {
	let ru_data = fs::read_to_string("data/item_names_ru.json")?;
	let ru_names: HashMap<String, String> = serde_json::from_str(&ru_data)?;

	for table in &["items", "artefacts", "resources"] {
		let rows: Vec<serde_json::Value> = DB.select(*table).await?;
		for row in &rows {
			if let Some(item_id) = row.get("item_id").and_then(|v| v.as_str()) {
				if let Some(ru_name) = ru_names.get(item_id) {
					DB.query(&format!("UPDATE {} SET label = $label WHERE item_id = $id", table))
						.bind(("label", ru_name.clone()))
						.bind(("id", item_id.to_string()))
						.await?;
				}
			}
		}
	}

	println!("Updated labels with Russian names");
	Ok(())
}

#[post("/api")]
async fn index_post(mut payload: Payload) -> Result<impl Responder, Box<dyn std::error::Error>> {
	// payload is a stream of Bytes objects
	let mut body = BytesMut::new();
	while let Some(chunk) = payload.next().await {
		let chunk = chunk?;
		// limit max size of in-memory payload
		if (body.len() + chunk.len()) > MAX_SIZE {
			println!("overflow")
		}
		body.extend_from_slice(&chunk);
	}
	let obj = serde_json::from_slice::<Data>(&body)?;
	let url = format!(
		"https://europe.albion-online-data.com/api/v2/stats/prices/{}.json?locations={}",
		&obj.items, obj.locations
	);
	let res = reqwest::get(url).await?.json::<Value>().await?;
	Ok(Json(res))
}

#[get("/api")]
async fn index_get() -> Result<impl Responder, Box<dyn std::error::Error>> {
	let url = String::from("https://europe.albion-online-data.com/api/v2/stats/prices/T4_LEATHER.json?locations=Thetford,Martlock,FortSterling,Bridgewatch,BlackMarket");

	let res = reqwest::get(url).await?.json::<Value>().await?;

	Ok(Json(res))
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	DB.connect::<Ws>("localhost:8000").await?;

	DB.signin(Root {
		username: "root".to_string(),
		password: "root".to_string(),
	})
	.await?;

	DB.use_ns("rust-api").use_db("rust-api").await?;

	DB.query("DEFINE TABLE items; DEFINE TABLE artefacts; DEFINE TABLE resources;").await?;
	DB.query("UPDATE items SET source = 'api' WHERE source = '' OR source = NONE;").await?;
	DB.query("UPDATE artefacts SET source = 'api' WHERE source = '' OR source = NONE;").await?;
	DB.query("UPDATE resources SET source = 'api' WHERE source = '' OR source = NONE;").await?;
	DB.query("UPDATE items SET updated_at = time::epoch(updated_at) WHERE updated_at CONTAINS 'T';").await?;
	DB.query("UPDATE artefacts SET updated_at = time::epoch(updated_at) WHERE updated_at CONTAINS 'T';").await?;
	DB.query("UPDATE resources SET updated_at = time::epoch(updated_at) WHERE updated_at CONTAINS 'T';").await?;
	DB.query("UPDATE items SET comment = '' WHERE comment IS NONE;").await?;
	DB.query("UPDATE items SET popularity = '0' WHERE popularity IS NONE;").await?;
	DB.query("UPDATE artefacts SET comment = '' WHERE comment IS NONE;").await?;
	DB.query("UPDATE artefacts SET popularity = '0' WHERE popularity IS NONE;").await?;
	DB.query("UPDATE resources SET comment = '' WHERE comment IS NONE;").await?;
	DB.query("UPDATE resources SET popularity = '0' WHERE popularity IS NONE;").await?;

	// Seed items and artefacts from item_recipes.json
	seed_items_from_recipes().await?;

	// Backfill resources on existing items that were created before resources field was added
	// Check if any items are missing resources
	let check: Vec<serde_json::Value> = DB.query("SELECT item_id, resources FROM items WHERE resources IS NONE LIMIT 1").await?.take(0)?;
	if !check.is_empty() {
		let data = fs::read_to_string("data/item_recipes.json")?;
		let recipes: Vec<Recipe> = serde_json::from_str(&data)?;
		let mut recipe_map: HashMap<String, Vec<RecipeResource>> = HashMap::new();
		for recipe in &recipes {
			let base_id = recipe.item_id.split('@').next().unwrap_or(&recipe.item_id);
			recipe_map.entry(base_id.to_string()).or_default();
			let entry = recipe_map.get_mut(base_id).unwrap();
			for res in &recipe.resources {
				if !entry.iter().any(|r| r.item_id == res.item_id) {
					entry.push(RecipeResource { item_id: res.item_id.clone(), count: res.count.clone() });
				}
			}
		}
		let existing_items: Vec<serde_json::Value> = DB.select("items").await?;
		for item in &existing_items {
			if let Some(item_id) = item.get("item_id").and_then(|v| v.as_str()) {
				if let Some(resources) = recipe_map.get(item_id) {
					let resources_str = serde_json::to_string(resources).unwrap_or_else(|_| "[]".to_string());
					DB.query("UPDATE items SET resources = $resources WHERE item_id = $id AND resources IS NONE")
						.bind(("resources", resources_str))
						.bind(("id", item_id.to_string()))
						.await?;
				}
			}
		}
		println!("Backfilled resources on existing items");
	}

	// Update existing labels with Russian names from localization
	update_labels_with_ru_names().await?;

	println!("Starting server at http://127.0.0.1:8082");
	HttpServer::new(|| {
		let cors = Cors::default()
			.allowed_origin("http://127.0.0.1:3001")
			.allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b"3001"))
			.allowed_origin("https://albion-online-market-monitoring.vercel.app")
			.allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
			.allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
			.allowed_header(http::header::CONTENT_TYPE)
			.max_age(3600);

		App::new()
			.wrap(cors)
			.service(index_get)
			.service(index_post)
			.service(items::create)
			.service(items::read)
			.service(items::update)
			.service(items::delete)
			.service(items::list)
			.service(artefacts::create)
			.service(artefacts::read)
			.service(artefacts::update)
			.service(artefacts::delete)
			.service(artefacts::list)
			.service(resources::create)
			.service(resources::read)
			.service(resources::update)
			.service(resources::delete)
			.service(resources::list)
			.service(prices::sync_prices)
			.service(prices::sync_status)
	})
	.bind(("127.0.0.1", 8082))?
	.run()
	.await?;

	Ok(())
}
