use dotenv::dotenv;
use std::env;
use tracing::warn;

pub fn check_required_env_vars() {
    dotenv().ok();
    let required_vars = [
        "QMAP_KEY",
        "DATABASE_URI",
        "BACKUP_PATH",
        "ALI_ACCESS_KEY_ID",
        "ALI_ACCESS_KEY_SECRET",
        "ALI_OSS_REGION",
        "ALI_OSS_ENDPOINT",
        "ALI_OSS_BUCKET_NAME",
    ];

    for var in required_vars {
        if let Err(_) = env::var(var) {
            warn!("缺少环境变量：{}", var);
        }
    }
}

pub fn database_uri() -> String {
    env::var("DATABASE_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
}

pub fn backup_path() -> String {
    env::var("BACKUP_PATH").unwrap_or_else(|_| "".to_string())
}

pub fn qmap_key() -> String {
    env::var("QMAP_KEY").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_acc_key_id() -> String {
    env::var("ALI_ACCESS_KEY_ID").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_acc_key_secret() -> String {
    env::var("ALI_ACCESS_KEY_SECRET").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_oss_region() -> String {
    env::var("ALI_OSS_REGION").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_oss_endpoint() -> String {
    env::var("ALI_OSS_ENDPOINT").unwrap_or_else(|_| "".to_string())
}

pub fn aliyun_oss_bucket_name() -> String {
    env::var("ALI_OSS_BUCKET_NAME").unwrap_or_else(|_| "".to_string())
}
pub const DB_NAME: &str = "maimap";
