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

const RESOURCES: &str = "resources";

#[derive(Serialize, Deserialize, surrealdb::types::SurrealValue)]
pub struct Resource {
	label: String,
	item_id: String,
	craft_price: String,
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

#[post("/api/resource/create")]
pub async fn create(resource: Json<Resource>) -> Result<Json<Option<Resource>>, Error> {
	let resource = DB.create(RESOURCES).content(resource.into_inner()).await?;
	Ok(Json(resource))
}

#[get("/api/resource/{item_id}")]
pub async fn read(item_id: Path<String>) -> Result<Json<Option<Resource>>, Error> {
	let mut result = DB.query("SELECT * FROM resources WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.await?;
	let resource: Option<Resource> = result.take(0)?;
	Ok(Json(resource))
}

#[put("/api/resource/{item_id}")]
pub async fn update(item_id: Path<String>, resource: Json<Resource>) -> Result<Json<Option<Resource>>, Error> {
	let mut result = DB.query("UPDATE resources SET label=$content.label, craft_price=$content.craft_price, sell_price_fort_sterling=$content.sell_price_fort_sterling, sell_price_martlock=$content.sell_price_martlock, sell_price_thetford=$content.sell_price_thetford, sell_price_brecilien=$content.sell_price_brecilien, buy_price_fort_sterling=$content.buy_price_fort_sterling, buy_price_martlock=$content.buy_price_martlock, buy_price_thetford=$content.buy_price_thetford, buy_price_brecilien=$content.buy_price_brecilien, orders_thetford=$content.orders_thetford, orders_fort_sterling=$content.orders_fort_sterling, orders_martlock=$content.orders_martlock, orders_brecilien=$content.orders_brecilien, created_at=$content.created_at, updated_at=$content.updated_at WHERE item_id = $id")
		.bind(("id", item_id.to_string()))
		.bind(("content", resource.into_inner()))
		.await?;
	let resource: Option<Resource> = result.take(0)?;
	Ok(Json(resource))
}

#[delete("/api/resource/{id}")]
pub async fn delete(id: Path<String>) -> Result<Json<Option<Resource>>, Error> {
	let mut result = DB.query("DELETE resources WHERE item_id = $id")
		.bind(("id", id.to_string()))
		.await?;
	let resource: Option<Resource> = result.take(0)?;
	Ok(Json(resource))
}

#[get("/api/resources")]
pub async fn list() -> Result<Json<Vec<Resource>>, Error> {
	let mut resources: Vec<Resource> = DB.select(RESOURCES).await?;
	resources.sort_by(|a, b| {
		a.label.cmp(&b.label)
			.then_with(|| tier(&a.item_id).cmp(&tier(&b.item_id)))
	});
	Ok(Json(resources))
}
