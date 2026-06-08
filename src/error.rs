use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("database error")]
	Db,
	#[error("request error")]
	Request,
}

impl ResponseError for Error {
	fn error_response(&self) -> HttpResponse {
		match self {
			Error::Db => HttpResponse::InternalServerError().body(self.to_string()),
			Error::Request => HttpResponse::InternalServerError().body(self.to_string()),
		}
	}
}

impl From<surrealdb::Error> for Error {
	fn from(error: surrealdb::Error) -> Self {
		eprintln!("{error}");
		Self::Db
	}
}

impl From<reqwest::Error> for Error {
	fn from(error: reqwest::Error) -> Self {
		eprintln!("{error}");
		Self::Request
	}
}
