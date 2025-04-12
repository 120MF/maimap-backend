use dotenv::dotenv;
use std::env;

pub fn database_uri() -> String {
    dotenv().ok();
    env::var("DATABASE_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
}

pub fn backup_path() -> String {
    dotenv().ok();
    env::var("BACKUP_PATH").unwrap_or_else(|_| "".to_string())
}

pub fn qmap_key() -> String {
    dotenv().ok();
    env::var("QMAP_KEY").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_acc_key_id() -> String {
    dotenv().ok();
    env::var("ALI_ACCESS_KEY_ID").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_acc_key_secret() -> String {
    dotenv().ok();
    env::var("ALI_ACCESS_KEY_SECRET").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_oss_region() -> String {
    dotenv().ok();
    env::var("ALI_OSS_REGION").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_oss_endpoint() -> String {
    dotenv().ok();
    env::var("ALI_OSS_ENDPOINT").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_oss_bucket_name() -> String {
    dotenv().ok();
    env::var("ALI_OSS_BUCKET_NAME").unwrap_or_else(|_| "".to_string())
}
pub const DB_NAME: &str = "maimap";
