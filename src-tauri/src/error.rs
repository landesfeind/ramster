use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Database error: {0}")]
	DatabaseError(String),
	#[error("SQL error: {0}")]
	SqlxError(#[from] sqlx::Error)
}	

pub type Result<T> = std::result::Result<T, Error>;

