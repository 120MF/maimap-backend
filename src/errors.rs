use mongodb::error::Error as MongoError;
use std::io;
use std::process::ExitStatus;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误：{0}")]
    Database(#[from] MongoError),

    #[error("备份执行失败：{0}")]
    BackupExecution(String),

    #[error("时间戳生成错误：{0}")]
    TimestampGeneration(String),

    #[error("IO错误：{0}")]
    Io(#[from] io::Error),

    #[error("OSS操作错误：{0}")]
    OssOperation(String),

    #[error("参数验证错误：{0}")]
    Validation(String),

    #[error("配置错误：{0}")]
    Configuration(String),

    #[error("命令执行错误：{status}, {stderr}")]
    CommandExecution { status: ExitStatus, stderr: String },

    #[error("解析内容失败：{0}")]
    Parse(String),

    #[error("调用腾讯地图API解析地址失败：{0}")]
    Geocoder(String),
}
