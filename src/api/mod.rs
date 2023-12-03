use rocket_db_pools::Connection;

use crate::database::PocketDB;

use self::error::ApiError;

pub mod error;

#[get("/")]
pub async fn hello(mut db: Connection<PocketDB>) -> Result<String, ApiError> {
    let (text,): (String,) = sqlx::query_as("select 'Hello from DB!'")
        .fetch_one(&mut **db)
        .await
        .map_err(|e| ApiError::from(e))?;
    Ok(text)
}
