use crate::error::Error;
use crate::DB;
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put};
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

fn tier(item_id: &str) -> u8 {
	item_id.as_bytes().get(1)
		.and_then(|&b| if b.is_ascii_digit() { Some(b - b'0') } else { None })
		.unwrap_or(0)
}

const ITEMS: &str = "items";

#[derive(Serialize, Deserialize, surrealdb::types::SurrealValue, Clone, Debug)]
pub struct RecipeResource {
	pub item_id: String,
	pub count: String,
}

#[derive(Serialize, Deserialize, surrealdb::types::SurrealValue)]
pub struct Artefact {
	label: String,
	item_id: String,
	crafted_item_id: String,
	sell_price_fort_sterling: String,
	sell_price_martlock: String,
	sell_price_thetford: String,
	sell_price_brecilien: String,
	buy_price_fort_sterling: String,
	buy_price_martlock: String,
	buy_price_thetford: String,
	buy_price_brecilien: String,
	orders_thetford: String,
	orders_fort_sterling: String,
	orders_martlock: String,
	orders_brecilien: String,
	created_at: String,
	updated_at: String,
	source: String,
}

#[derive(Serialize, Deserialize, surrealdb::types::SurrealValue)]
pub struct Item {
	label: String,
	item_id: String,
	craft_price: String,
	enchantment_price: String,
	artefact_id: String,
	sell_price_fort_sterling: String,
	sell_price_martlock: String,
	sell_price_thetford: String,
	sell_price_brecilien: String,
	buy_price_fort_sterling: String,
	buy_price_martlock: String,
	buy_price_thetford: String,
	buy_price_brecilien: String,
	orders_thetford: String,
	orders_fort_sterling: String,
	orders_martlock: String,
	orders_brecilien: String,
	created_at: String,
	updated_at: String,
	artefact: Option<Artefact>,
	#[serde(default)]
	resources: String,
	source: String,
	#[serde(default)]
	comment: String,
	#[serde(default)]
	popularity: String,
}

#[derive(Serialize, Deserialize, surrealdb::types::SurrealValue)]
pub struct ItemWithoutArtefact {
	label: String,
	item_id: String,
	craft_price: String,
	enchantment_price: String,
	artefact_id: String,
	sell_price_fort_sterling: String,
	sell_price_martlock: String,
	sell_price_thetford: String,
	sell_price_brecilien: String,
	buy_price_fort_sterling: String,
	buy_price_martlock: String,
	buy_price_thetford: String,
	buy_price_brecilien: String,
	orders_thetford: String,
	orders_fort_sterling: String,
	orders_martlock: String,
	orders_brecilien: String,
	created_at: String,
	updated_at: String,
	#[serde(default)]
	resources: String,
	source: String,
	#[serde(default)]
	comment: String,
	#[serde(default)]
	popularity: String,
}

#[derive(Deserialize)]
pub struct ItemUpdate {
	label: String,
	item_id: String,
	craft_price: String,
	enchantment_price: String,
	artefact_id: String,
	sell_price_fort_sterling: String,
	sell_price_martlock: String,
	sell_price_thetford: String,
	sell_price_brecilien: String,
	buy_price_fort_sterling: String,
	buy_price_martlock: String,
	buy_price_thetford: String,
	buy_price_brecilien: String,
	orders_thetford: String,
	orders_fort_sterling: String,
	orders_martlock: String,
	orders_brecilien: String,
	created_at: String,
	updated_at: String,
	#[serde(default)]
	resources: serde_json::Value,
	source: String,
	#[serde(default)]
	comment: String,
	#[serde(default)]
	popularity: String,
}

#[derive(Serialize)]
pub struct ItemResponse {
	label: String,
	item_id: String,
	craft_price: String,
	enchantment_price: String,
	artefact_id: String,
	sell_price_fort_sterling: String,
	sell_price_martlock: String,
	sell_price_thetford: String,
	sell_price_brecilien: String,
	buy_price_fort_sterling: String,
	buy_price_martlock: String,
	buy_price_thetford: String,
	buy_price_brecilien: String,
	orders_thetford: String,
	orders_fort_sterling: String,
	orders_martlock: String,
	orders_brecilien: String,
	created_at: String,
	updated_at: String,
	artefact: Option<Artefact>,
	resources: serde_json::Value,
	source: String,
	comment: String,
	popularity: String,
}

impl ItemResponse {
	pub fn from_item(item: Item) -> Self {
		let resources: serde_json::Value = serde_json::from_str(&item.resources).unwrap_or(serde_json::Value::Array(vec![]));
		Self {
			label: item.label,
			item_id: item.item_id,
			craft_price: item.craft_price,
			enchantment_price: item.enchantment_price,
			artefact_id: item.artefact_id,
			sell_price_fort_sterling: item.sell_price_fort_sterling,
			sell_price_martlock: item.sell_price_martlock,
			sell_price_thetford: item.sell_price_thetford,
			sell_price_brecilien: item.sell_price_brecilien,
			buy_price_fort_sterling: item.buy_price_fort_sterling,
			buy_price_martlock: item.buy_price_martlock,
			buy_price_thetford: item.buy_price_thetford,
			buy_price_brecilien: item.buy_price_brecilien,
			orders_thetford: item.orders_thetford,
			orders_fort_sterling: item.orders_fort_sterling,
			orders_martlock: item.orders_martlock,
			orders_brecilien: item.orders_brecilien,
			created_at: item.created_at,
			updated_at: item.updated_at,
			artefact: item.artefact,
			resources,
			source: item.source,
			comment: item.comment,
			popularity: item.popularity,
		}
	}
}

#[post("/api/item/create")]
pub async fn create(item: Json<Item>) -> Result<Json<Option<Item>>, Error> {
	let item = DB.create(ITEMS).content(item.into_inner()).await?;
	Ok(Json(item))
}

#[get("/api/item/{item_id}")]
pub async fn read(item_id: Path<String>) -> Result<Json<Option<ItemResponse>>, Error> {
	let mut result = DB.query("SELECT label, item_id, craft_price, enchantment_price, artefact_id, sell_price_fort_sterling, sell_price_martlock, sell_price_thetford, sell_price_brecilien, buy_price_fort_sterling, buy_price_martlock, buy_price_thetford, buy_price_brecilien, orders_thetford, orders_fort_sterling, orders_martlock, orders_brecilien, <string>created_at AS created_at, <string>updated_at AS updated_at, resources, source, comment, popularity FROM items WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.await?;
	let item: Option<Item> = result.take(0)?;
	Ok(Json(item.map(ItemResponse::from_item)))
}

#[put("/api/item/{item_id}")]
pub async fn update(item_id: Path<String>, item: Json<ItemUpdate>) -> Result<Json<Option<ItemResponse>>, Error> {
	let input = item.into_inner();
	let resources_str = serde_json::to_string(&input.resources).unwrap_or_else(|_| "[]".to_string());
	let mut result = DB.query("UPDATE items SET label=$label, craft_price=$craft_price, enchantment_price=$enchantment_price, artefact_id=$artefact_id, sell_price_fort_sterling=$sell_price_fort_sterling, sell_price_martlock=$sell_price_martlock, sell_price_thetford=$sell_price_thetford, sell_price_brecilien=$sell_price_brecilien, buy_price_fort_sterling=$buy_price_fort_sterling, buy_price_martlock=$buy_price_martlock, buy_price_thetford=$buy_price_thetford, buy_price_brecilien=$buy_price_brecilien, orders_thetford=$orders_thetford, orders_fort_sterling=$orders_fort_sterling, orders_martlock=$orders_martlock, orders_brecilien=$orders_brecilien, created_at=$created_at, updated_at=$updated_at, resources=$resources, comment=$comment, popularity=$popularity, source='manual' WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.bind(("label", input.label))
		.bind(("craft_price", input.craft_price))
		.bind(("enchantment_price", input.enchantment_price))
		.bind(("artefact_id", input.artefact_id))
		.bind(("sell_price_fort_sterling", input.sell_price_fort_sterling))
		.bind(("sell_price_martlock", input.sell_price_martlock))
		.bind(("sell_price_thetford", input.sell_price_thetford))
		.bind(("sell_price_brecilien", input.sell_price_brecilien))
		.bind(("buy_price_fort_sterling", input.buy_price_fort_sterling))
		.bind(("buy_price_martlock", input.buy_price_martlock))
		.bind(("buy_price_thetford", input.buy_price_thetford))
		.bind(("buy_price_brecilien", input.buy_price_brecilien))
		.bind(("orders_thetford", input.orders_thetford))
		.bind(("orders_fort_sterling", input.orders_fort_sterling))
		.bind(("orders_martlock", input.orders_martlock))
		.bind(("orders_brecilien", input.orders_brecilien))
		.bind(("created_at", input.created_at))
		.bind(("updated_at", input.updated_at))
		.bind(("resources", resources_str))
		.bind(("comment", input.comment))
		.bind(("popularity", input.popularity))
		.await?;
	let item: Option<Item> = result.take(0)?;
	Ok(Json(item.map(ItemResponse::from_item)))
}

#[delete("/api/item/{id}")]
pub async fn delete(id: Path<String>) -> Result<Json<Option<Item>>, Error> {
	let mut result = DB.query("DELETE items WHERE item_id = $id")
		.bind(("id", id.to_string()))
		.await?;
	let item: Option<Item> = result.take(0)?;
	Ok(Json(item))
}

#[get("/api/items")]
pub async fn list() -> Result<Json<Vec<ItemResponse>>, Error> {
	let mut result = DB.query("SELECT label, item_id, craft_price, enchantment_price, artefact_id, sell_price_fort_sterling, sell_price_martlock, sell_price_thetford, sell_price_brecilien, buy_price_fort_sterling, buy_price_martlock, buy_price_thetford, buy_price_brecilien, orders_thetford, orders_fort_sterling, orders_martlock, orders_brecilien, <string>created_at AS created_at, <string>updated_at AS updated_at, resources, source, comment, popularity FROM items")
		.await?;
	let mut items: Vec<Item> = result.take(0)?;
	items.sort_by(|a, b| {
		a.label.cmp(&b.label)
			.then_with(|| tier(&a.item_id).cmp(&tier(&b.item_id)))
	});
	Ok(Json(items.into_iter().map(ItemResponse::from_item).collect()))
}
