use crate::error::Error;
use crate::DB;
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put};
use serde::{Deserialize, Serialize};

const ITEMS: &str = "items";

#[derive(Serialize, Deserialize)]
pub struct Artefact {
	label: String,
	item_id: String,
	crafted_item_id: String,
	sell_price_fort_sterling: String,
	sell_price_martlock: String,
	sell_price_thetford: String,
	buy_price_fort_sterling: String,
	buy_price_martlock: String,
	buy_price_thetford: String,
	orders_thetford: String,
	orders_fort_sterling: String,
	orders_martlock: String,
	created_at: String,
	updated_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct Item {
	label: String,
	item_id: String,
	craft_price: String,
	enchantment_price: String,
	artefact_id: String,
	sell_price_fort_sterling: String,
	sell_price_martlock: String,
	sell_price_thetford: String,
	buy_price_fort_sterling: String,
	buy_price_martlock: String,
	buy_price_thetford: String,
	orders_thetford: String,
	orders_fort_sterling: String,
	orders_martlock: String,
	created_at: String,
	updated_at: String,
	artefact: Option<Artefact>,
}

#[post("/api/item/create")]
pub async fn create(item: Json<Item>) -> Result<Json<Option<Item>>, Error> {
	let item = DB.create(ITEMS).content(item).await?;
	Ok(Json(item))
}

#[get("/api/item/{item_id}")]
pub async fn read(item_id: Path<String>) -> Result<Json<Option<Item>>, Error> {
	let sql = "SELECT
									    label,
									    item_id,
									    craft_price,
											enchantment_price,
									    sell_price_fort_sterling,
									    sell_price_martlock,
									    sell_price_thetford,
									    buy_price_fort_sterling,
									    buy_price_martlock,
									    buy_price_thetford,
									    orders_thetford,
									    orders_fort_sterling,
									    orders_martlock,
									    created_at,
									    updated_at,
									    artefact,
											artefact_id
									FROM (
									SELECT *,
									(SELECT * FROM artefacts ) as artefact
									FROM items WHERE item_id = $id
									SPLIT artefact
									)
									WHERE artefact_id = artefact.item_id ORDER BY item_id ASC";
	let item = DB.query(sql)
		.bind(("id", &*item_id))
		.bind(("table", ITEMS))
		.await?.take(0)?;

	Ok(Json(item))
}

#[derive(Serialize)]
pub struct ItemContent {
	content: Json<Item>,
}

#[put("/api/item/{item_id}")]
pub async fn update(item_id: Path<String>, item: Json<Item>) -> Result<Json<Option<Item>>, Error> {
	let sql = [
		"UPDATE items CONTENT $content WHERE item_id = '",
		&item_id,
		"'",
	]
	.concat();

	let item = DB
		.query(sql)
		.bind(ItemContent { content: item })
		.bind(("table", ITEMS))
		.await?
		.take(0)?;

	Ok(Json(item))
}

#[delete("/api/item/{id}")]
pub async fn delete(id: Path<String>) -> Result<Json<Option<Item>>, Error> {
	let item = DB.delete((ITEMS, &*id)).await?;
	Ok(Json(item))
}

#[get("/api/items")]
pub async fn list() -> Result<Json<Vec<Item>>, Error> {
	let sql = "SELECT
									    label,
									    item_id,
									    craft_price,
											enchantment_price,
									    sell_price_fort_sterling,
									    sell_price_martlock,
									    sell_price_thetford,
									    buy_price_fort_sterling,
									    buy_price_martlock,
									    buy_price_thetford,
									    orders_thetford,
									    orders_fort_sterling,
									    orders_martlock,
									    created_at,
									    updated_at,
									    artefact,
											artefact_id
									FROM (
									SELECT *,
									(SELECT * FROM artefacts) as artefact
									FROM items
									SPLIT artefact
									)
									WHERE artefact_id = artefact.item_id ORDER BY item_id ASC";
	let items = DB.query(sql).bind(("table", ITEMS)).await?.take(0)?;

	Ok(Json(items))
}
