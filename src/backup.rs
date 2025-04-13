use crate::env::{
    DB_NAME, aliyun_acc_key_id, aliyun_acc_key_secret, aliyun_oss_bucket_name, aliyun_oss_endpoint,
    aliyun_oss_region, backup_path, database_uri,
};
use crate::errors::AppError;
use ali_oss_rs::Client;
use ali_oss_rs::object::ObjectOperations;
use anyhow::{Context, Result};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub async fn backup_database() -> Result<String> {
    let backup_dir = backup_path();
    std::fs::create_dir_all(&backup_dir).context("创建备份目录失败")?;

    let client = Client::new(
        aliyun_acc_key_id(),
        aliyun_acc_key_secret(),
        aliyun_oss_region(),
        aliyun_oss_endpoint(),
    );

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::TimestampGeneration(e.to_string()))?
        .as_secs();

    let filename = format!("maimap_{}.gz", timestamp);
    let filepath = format!("{}{}", backup_dir, filename);
    info!("备份文件：{}", filepath);
    let output = Command::new("mongodump")
        .arg(format!("--uri={}", database_uri()))
        .arg(format!("--db={}", DB_NAME))
        .arg("--gzip")
        .arg(format!("--archive={}", filepath))
        .output()
        .context("执行备份命令失败")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::CommandExecution {
            status: output.status,
            stderr,
        }
        .into());
    }

    client
        .put_object_from_file(aliyun_oss_bucket_name(), &filename, filepath, None)
        .await
        .map_err(|e| AppError::OssOperation(e.to_string()))?;

    Ok(filename)
}
