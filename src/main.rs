#![allow(dead_code)]
mod error;
mod items;
mod artefacts;
mod person;

use actix_cors::Cors;
use actix_web::web::{BytesMut, Json, Payload};
use actix_web::{get, http, post, App, HttpServer, Responder, Result};
use futures::StreamExt;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

static DB: Surreal<Client> = Surreal::init();

#[derive(Serialize, Deserialize, Debug)]
struct Data {
	items: String,
	locations: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

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
		"https://east.albion-online-data.com/api/v2/stats/prices/{}.json?locations={}",
		&obj.items, obj.locations
	);
	let res = reqwest::get(url).await?.json::<Value>().await?;
	Ok(Json(res))
}

#[get("/api")]
async fn index_get() -> Result<impl Responder, Box<dyn std::error::Error>> {
	let url = String::from("https://east.albion-online-data.com/api/v2/stats/prices/T4_LEATHER.json?locations=Thetford,Martlock,FortSterling,Bridgewatch,BlackMarket");

	let res = reqwest::get(url).await?.json::<Value>().await?;

	Ok(Json(res))
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	DB.connect::<Ws>("localhost:8000").await?;

	DB.signin(Root {
		username: "root",
		password: "root",
	})
	.await?;

	DB.use_ns("rust-api").use_db("rust-api").await?;

	println!("Starting server at http://127.0.0.1:8080");
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
	})
	.bind(("127.0.0.1", 8080))?
	.run()
	.await?;

	Ok(())
}
