use crate::error::Error;
use crate::DB;
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put};
use serde::{Deserialize, Serialize};

const ARTEFACTS: &str = "artefacts";

#[derive(Serialize, Deserialize)]
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

#[post("/api/artefact/create")]
pub async fn create(artefact: Json<Artefact>) -> Result<Json<Option<Artefact>>, Error> {
	let artefact = DB.create(ARTEFACTS).content(artefact).await?;
	Ok(Json(artefact))
}

#[get("/api/artefact/{item_id}")]
pub async fn read(item_id: Path<String>) -> Result<Json<Option<Artefact>>, Error> {
	let item = DB.query("SELECT * FROM artefacts WHERE item_id = $id")
		.bind(("id", &*item_id))
		.bind(("table", ARTEFACTS))
		.await?.take(0)?;

	Ok(Json(item))
}

#[derive(Serialize)]
pub struct ArtefactContent {
	content: Json<Artefact>,
}

#[put("/api/artefact/{item_id}")]
pub async fn update(item_id: Path<String>, artefact: Json<Artefact>) -> Result<Json<Option<Artefact>>, Error> {
	let sql = [
		"UPDATE artefacts CONTENT $content WHERE item_id = '",
		&item_id,
		"'",
	]
	.concat();

	let item = DB
		.query(sql)
		.bind(ArtefactContent { content: artefact })
		.bind(("table", ARTEFACTS))
		.await?
		.take(0)?;

	Ok(Json(item))
}

#[delete("/api/artefact/{id}")]
pub async fn delete(id: Path<String>) -> Result<Json<Option<Artefact>>, Error> {
	let artefact = DB.delete((ARTEFACTS, &*id)).await?;
	Ok(Json(artefact))
}

#[get("/api/artefacts")]
pub async fn list() -> Result<Json<Vec<Artefact>>, Error> {
	let sql = "SELECT * FROM artefacts ORDER BY item_id ASC";
	let artefacts = DB.query(sql).bind(("table", ARTEFACTS)).await?.take(0)?;
	Ok(Json(artefacts))
}
