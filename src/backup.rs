use crate::env::{
    DB_NAME, aliyun_acc_key_id, aliyun_acc_key_secret, aliyun_oss_bucket_name, aliyun_oss_endpoint,
    aliyun_oss_region, backup_path, database_uri,
};
use ali_oss_rs::Client;
use ali_oss_rs::object::ObjectOperations;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub async fn backup_database() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(
        aliyun_acc_key_id(),
        aliyun_acc_key_secret(),
        aliyun_oss_region(),
        aliyun_oss_endpoint(),
    );

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("获取时间失败：{}", e))?
        .as_secs();

    let filename = format!("maimap_{}.gz", timestamp);
    let filepath = format!("{}{}", backup_path(), filename);
    info!("备份文件：{}", filepath);
    let output = Command::new("mongodump")
        .arg(format!("--uri={}", database_uri()))
        .arg(format!("--db={}", DB_NAME))
        .arg("--gzip")
        .arg(format!("--archive={}", filepath))
        .output()?;
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Box::<dyn std::error::Error>::from(format!(
            "备份失败：{}",
            error_msg
        )));
    }

    client
        .put_object_from_file(aliyun_oss_bucket_name(), &filename, filepath, None)
        .await
        .map_err(|e| format!("上传到OSS失败：{}", e))?;

    Ok(filename)
}
