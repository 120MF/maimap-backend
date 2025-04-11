use crate::env::{DB_NAME, backup_path, database_uri};
use std::process::Command;

pub fn backup_database() -> Result<(), String> {
    let output = Command::new("mongodump")
        .arg(format!("--uri={}", database_uri()))
        .arg(format!("--db={}", DB_NAME))
        .arg("--gzip")
        .arg(format!("--archive={}maimap.gz", backup_path()))
        .output()
        .map_err(|e| format!("执行备份任务失败：{}", e))?;
    if output.status.success() {
        println!("数据库备份成功：{}maimap.gz", backup_path());
        Ok(())
    } else {
        Err(format!(
            "备份失败：{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
