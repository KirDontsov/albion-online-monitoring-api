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
}

#[post("/api/item/create")]
pub async fn create(item: Json<Item>) -> Result<Json<Option<Item>>, Error> {
	let item = DB.create(ITEMS).content(item.into_inner()).await?;
	Ok(Json(item))
}

#[get("/api/item/{item_id}")]
pub async fn read(item_id: Path<String>) -> Result<Json<Option<Item>>, Error> {
	let mut result = DB.query("SELECT * FROM items WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.await?;
	let item: Option<Item> = result.take(0)?;
	Ok(Json(item))
}

#[put("/api/item/{item_id}")]
pub async fn update(item_id: Path<String>, item: Json<ItemWithoutArtefact>) -> Result<Json<Option<Item>>, Error> {
	let mut result = DB.query("UPDATE items SET label=$content.label, craft_price=$content.craft_price, enchantment_price=$content.enchantment_price, artefact_id=$content.artefact_id, sell_price_fort_sterling=$content.sell_price_fort_sterling, sell_price_martlock=$content.sell_price_martlock, sell_price_thetford=$content.sell_price_thetford, sell_price_brecilien=$content.sell_price_brecilien, buy_price_fort_sterling=$content.buy_price_fort_sterling, buy_price_martlock=$content.buy_price_martlock, buy_price_thetford=$content.buy_price_thetford, buy_price_brecilien=$content.buy_price_brecilien, orders_thetford=$content.orders_thetford, orders_fort_sterling=$content.orders_fort_sterling, orders_martlock=$content.orders_martlock, orders_brecilien=$content.orders_brecilien, created_at=$content.created_at, updated_at=$content.updated_at WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.bind(("content", item.into_inner()))
		.await?;
	let item: Option<Item> = result.take(0)?;
	Ok(Json(item))
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
pub async fn list() -> Result<Json<Vec<Item>>, Error> {
	let mut items: Vec<Item> = DB.select(ITEMS).await?;
	items.sort_by(|a, b| {
		a.label.cmp(&b.label)
			.then_with(|| tier(&a.item_id).cmp(&tier(&b.item_id)))
	});
	Ok(Json(items))
}
