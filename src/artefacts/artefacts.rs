use crate::error::Error;
use crate::DB;
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put};
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

const ARTEFACTS: &str = "artefacts";

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

#[post("/api/artefact/create")]
pub async fn create(artefact: Json<Artefact>) -> Result<Json<Option<Artefact>>, Error> {
	let artefact = DB.create(ARTEFACTS).content(artefact.into_inner()).await?;
	Ok(Json(artefact))
}

#[get("/api/artefact/{item_id}")]
pub async fn read(item_id: Path<String>) -> Result<Json<Option<Artefact>>, Error> {
	let mut result = DB.query("SELECT * FROM artefacts WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.await?;
	let item: Option<Artefact> = result.take(0)?;
	Ok(Json(item))
}

#[put("/api/artefact/{item_id}")]
pub async fn update(item_id: Path<String>, artefact: Json<Artefact>) -> Result<Json<Option<Artefact>>, Error> {
	let mut result = DB.query("UPDATE artefacts SET label=$content.label, crafted_item_id=$content.crafted_item_id, sell_price_fort_sterling=$content.sell_price_fort_sterling, sell_price_martlock=$content.sell_price_martlock, sell_price_thetford=$content.sell_price_thetford, sell_price_brecilien=$content.sell_price_brecilien, buy_price_fort_sterling=$content.buy_price_fort_sterling, buy_price_martlock=$content.buy_price_martlock, buy_price_thetford=$content.buy_price_thetford, buy_price_brecilien=$content.buy_price_brecilien, orders_thetford=$content.orders_thetford, orders_fort_sterling=$content.orders_fort_sterling, orders_martlock=$content.orders_martlock, orders_brecilien=$content.orders_brecilien, created_at=$content.created_at, updated_at=$content.updated_at, source='manual' WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.bind(("content", artefact.into_inner()))
		.await?;
	let item: Option<Artefact> = result.take(0)?;
	Ok(Json(item))
}

#[delete("/api/artefact/{id}")]
pub async fn delete(id: Path<String>) -> Result<Json<Option<Artefact>>, Error> {
	let mut result = DB.query("DELETE artefacts WHERE item_id = $id")
		.bind(("id", id.to_string()))
		.await?;
	let item: Option<Artefact> = result.take(0)?;
	Ok(Json(item))
}

fn tier(item_id: &str) -> u8 {
	item_id.as_bytes().get(1)
		.and_then(|&b| if b.is_ascii_digit() { Some(b - b'0') } else { None })
		.unwrap_or(0)
}

#[get("/api/artefacts")]
pub async fn list() -> Result<Json<Vec<Artefact>>, Error> {
	let mut artefacts: Vec<Artefact> = DB.select(ARTEFACTS).await?;
	artefacts.sort_by(|a, b| {
		a.label.cmp(&b.label)
			.then_with(|| tier(&a.item_id).cmp(&tier(&b.item_id)))
	});
	Ok(Json(artefacts))
}
