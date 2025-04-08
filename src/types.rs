use mongodb::bson::oid::ObjectId;
use mongodb::bson::{DateTime, Decimal128};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Arcade {
    pub _id: Option<ObjectId>,
    /// 机厅地址
    pub arcade_address: String,
    /// 单局花销
    pub arcade_cost: Option<f64>,
    /// 机台数量
    pub arcade_count: Option<i32>,
    /// 机厅存活情况
    pub arcade_dead: bool,
    /// 机厅ID
    pub arcade_id: i32,
    /// 机厅纬度
    pub arcade_lat: Decimal128,
    /// 机厅经度
    pub arcade_lng: Decimal128,
    /// 机厅名
    pub arcade_name: String,
    /// 创建时间
    pub created_at: DateTime,
}
