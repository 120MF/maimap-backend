use dotenv::dotenv;
use std::env;

pub fn database_uri() -> String {
    dotenv().ok();
    env::var("DATABASE_URI").unwrap_or("mongodb://localhost:27017".to_string())
}

pub fn backup_path() -> String {
    dotenv().ok();
    env::var("BACKUP_PATH").unwrap_or("".to_string())
}
pub const DB_NAME: &str = "maimap";
