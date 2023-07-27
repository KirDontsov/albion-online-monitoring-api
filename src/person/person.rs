use crate::error::Error;
use crate::DB;
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put};
use serde::{Deserialize, Serialize};

const PERSON: &str = "person";

#[derive(Serialize, Deserialize)]
pub struct Person {
	name: String,
	age: String,
}

#[post("/api/person/create")]
pub async fn create(person: Json<Person>) -> Result<Json<Option<Person>>, Error> {
	let person = DB.create(PERSON).content(person).await?;
	Ok(Json(person))
}

#[get("/api/person/{name}")]
pub async fn read(name: Path<String>) -> Result<Json<Option<Person>>, Error> {
	let sql = ["SELECT * FROM person WHERE name = '", &name, "'"].concat();

	let person = DB.query(sql).bind(("table", PERSON)).await?.take(0)?;

	Ok(Json(person))
}

#[derive(Serialize)]
pub struct PersonContent {
	content: Json<Person>,
}

#[put("/api/person/{name}")]
pub async fn update(
	name: Path<String>,
	person: Json<Person>,
) -> Result<Json<Option<Person>>, Error> {
	let sql = ["UPDATE person CONTENT $content WHERE name = '", &name, "'"].concat();

	let person = DB
		.query(sql)
		.bind(PersonContent { content: person })
		.bind(("table", PERSON))
		.await?
		.take(0)?;

	Ok(Json(person))
}

#[delete("/api/person/{id}")]
pub async fn delete(id: Path<String>) -> Result<Json<Option<Person>>, Error> {
	let person = DB.delete((PERSON, &*id)).await?;
	Ok(Json(person))
}

#[get("/api/people")]
pub async fn list() -> Result<Json<Vec<Person>>, Error> {
	let people: Vec<Person> = DB.select(PERSON).await?;
	Ok(Json(people))
}
