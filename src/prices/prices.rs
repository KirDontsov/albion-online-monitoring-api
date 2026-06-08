use crate::error::Error;
use crate::DB;
use actix_web::web::Json;
use actix_web::{get, post};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

const API_BASE: &str = "https://europe.albion-online-data.com";
const LOCATIONS: &str = "Thetford,Fort Sterling,Martlock,Brecilien";
const BATCH_SIZE: usize = 150;

#[derive(Serialize, Deserialize, Debug)]
struct AlbionPriceResponse {
	item_id: String,
	city: String,
	#[serde(default)]
	quality: u32,
	#[serde(default)]
	sell_price_min: u32,
	#[serde(default)]
	sell_price_max: u32,
	#[serde(default)]
	buy_price_min: u32,
	#[serde(default)]
	buy_price_max: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncStatus {
	pub last_sync: Option<u64>,
	pub items_updated: u32,
	pub items_skipped: u32,
	pub errors: u32,
	pub running: bool,
}

static SYNC_STATUS: LazyLock<std::sync::Mutex<SyncStatus>> = LazyLock::new(|| {
	std::sync::Mutex::new(SyncStatus {
		last_sync: None,
		items_updated: 0,
		items_skipped: 0,
		errors: 0,
		running: false,
	})
});

fn city_to_column(city: &str) -> Option<(&str, &str)> {
	match city {
		"Thetford" => Some(("sell_price_thetford", "buy_price_thetford")),
		"Fort Sterling" => Some(("sell_price_fort_sterling", "buy_price_fort_sterling")),
		"Martlock" => Some(("sell_price_martlock", "buy_price_martlock")),
		"Brecilien" => Some(("sell_price_brecilien", "buy_price_brecilien")),
		_ => None,
	}
}

async fn fetch_item_ids() -> Result<Vec<String>, Error> {
	let mut ids: Vec<String> = Vec::new();

	let items: Vec<serde_json::Value> = DB.select("items").await?;
	for item in &items {
		if let Some(id) = item.get("item_id").and_then(|v| v.as_str()) {
			ids.push(id.to_string());
		}
	}

	let artefacts: Vec<serde_json::Value> = DB.select("artefacts").await?;
	for a in &artefacts {
		if let Some(id) = a.get("item_id").and_then(|v| v.as_str()) {
			ids.push(id.to_string());
		}
	}

	let resources: Vec<serde_json::Value> = DB.select("resources").await?;
	for r in &resources {
		if let Some(id) = r.get("item_id").and_then(|v| v.as_str()) {
			ids.push(id.to_string());
		}
	}

	ids.dedup();
	println!("Found {} unique item IDs to sync", ids.len());
	Ok(ids)
}

async fn fetch_prices_batch(item_ids: &[String]) -> Result<Vec<AlbionPriceResponse>, Error> {
	let ids_str = item_ids.join(",");
	let url = format!(
		"{}/api/v2/stats/prices/{}.json?locations={}&qualities=1",
		API_BASE, ids_str, LOCATIONS
	);
	println!("Fetching prices from: {}", url);
	let client = reqwest::Client::new();
	let res = client.get(&url).send().await?.json::<Vec<AlbionPriceResponse>>().await?;
	println!("Got {} price records", res.len());
	Ok(res)
}

async fn update_db_prices(prices: &[AlbionPriceResponse]) -> Result<(u32, u32), Error> {
	let mut updated: u32 = 0;
	let mut skipped: u32 = 0;
	let one_day_ago = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs() as i64 - 86400;

	for price in prices {
		if price.sell_price_min == 0 {
			skipped += 1;
			continue;
		}
		if let Some((sell_col, buy_col)) = city_to_column(&price.city) {
			let tables = ["items", "artefacts", "resources"];
			for table in &tables {
				let query = format!(
					"UPDATE {} SET {} = $sell, {} = $buy, updated_at = $now WHERE item_id = $id AND (source = 'api' OR updated_at < $cutoff)",
					table, sell_col, buy_col
				);
				let now = SystemTime::now()
					.duration_since(UNIX_EPOCH)
					.unwrap()
					.as_secs() as i64;
				let mut result = DB
					.query(&query)
					.bind(("sell", price.sell_price_min.to_string()))
					.bind(("buy", price.buy_price_max.to_string()))
					.bind(("id", price.item_id.clone()))
					.bind(("now", now))
					.bind(("cutoff", one_day_ago))
					.await?;
				let _: Vec<serde_json::Value> = result.take(0)?;
				updated += 1;
			}
		}
	}

	Ok((updated, skipped))
}

#[post("/api/prices/sync")]
pub async fn sync_prices() -> Result<Json<SyncStatus>, Error> {
	{
		let mut status = SYNC_STATUS.lock().unwrap();
		if status.running {
			return Ok(Json(status.clone()));
		}
		status.running = true;
		status.items_updated = 0;
		status.items_skipped = 0;
		status.errors = 0;
	}

	let item_ids = match fetch_item_ids().await {
		Ok(ids) => ids,
		Err(e) => {
			let mut status = SYNC_STATUS.lock().unwrap();
			status.running = false;
			status.errors += 1;
			println!("Error fetching item IDs: {}", e);
			return Ok(Json(status.clone()));
		}
	};

	let mut total_updated: u32 = 0;
	let mut total_skipped: u32 = 0;
	let mut total_errors: u32 = 0;

	for chunk in item_ids.chunks(BATCH_SIZE) {
		println!("Processing batch of {} items...", chunk.len());
		match fetch_prices_batch(chunk).await {
			Ok(prices) => {
				match update_db_prices(&prices).await {
					Ok((u, s)) => {
						total_updated += u;
						total_skipped += s;
					}
					Err(e) => {
						total_errors += 1;
						println!("Error updating DB: {}", e);
					}
				}
			}
			Err(e) => {
				total_errors += 1;
				println!("Error fetching prices batch: {}", e);
			}
		}

		// Rate limit: wait between batches
		if chunk.len() == BATCH_SIZE {
			tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
		}
	}

	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();

	let mut status = SYNC_STATUS.lock().unwrap();
	status.last_sync = Some(now);
	status.items_updated = total_updated;
	status.items_skipped = total_skipped;
	status.errors = total_errors;
	status.running = false;

	Ok(Json(status.clone()))
}

#[get("/api/prices/status")]
pub async fn sync_status() -> Result<Json<SyncStatus>, Error> {
	let status = SYNC_STATUS.lock().unwrap();
	Ok(Json(status.clone()))
}
