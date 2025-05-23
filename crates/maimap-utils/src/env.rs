use dotenvy::from_path;
use std::env;
use std::path::PathBuf;
use tracing::warn;

fn find_env_file() -> Option<PathBuf> {
    // 尝试从当前目录及其父目录查找.env文件
    let mut current_dir = env::current_dir().ok()?;

    loop {
        let env_path = current_dir.join(".env");
        if env_path.exists() {
            return Some(env_path);
        }

        // 移动到上一级目录
        if !current_dir.pop() {
            break;
        }
    }

    None
}
pub fn check_required_env_vars() {
    // 查找并加载项目根目录的.env文件
    if let Some(env_path) = find_env_file() {
        match from_path(&env_path) {
            Ok(_) => tracing::info!("已加载环境变量文件: {:?}", env_path),
            Err(e) => warn!("无法加载环境变量文件 {:?}: {:?}", env_path, e),
        }
    } else {
        warn!("未找到.env文件");
    }

    let required_vars = [
        "QMAP_KEY",
        "FRONTEND_URL",
        "DATABASE_URI",
        "TEST_DATABASE_URI",
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
pub fn frontend_uri() -> String {
    env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

pub fn database_uri() -> String {
    env::var("DATABASE_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
}

pub fn test_database_uri() -> String {
    env::var("TEST_DATABASE_URI").unwrap().to_string()
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
