use crate::traits::ToResponse;
use maimap_derive::ToResponse;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{DateTime, Decimal128};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    pub r#type: String,
    pub coordinates: [f64; 2],
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write! {f, "{},{}",self.coordinates[0], self.coordinates[1]}
    }
}
impl Point {
    pub fn new(lng: f64, lat: f64) -> Self {
        Self {
            r#type: "Point".to_string(),
            coordinates: [lng, lat],
        }
    }
}

#[derive(Deserialize, Serialize, ToResponse)]
pub struct Arcade {
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

    #[DoNotRespond]
    pub arcade_pos: Option<Point>,

    /// 机厅名
    pub arcade_name: String,
    /// 创建时间
    pub created_at: DateTime,
}

#[derive(Serialize, Deserialize, ToResponse)]
pub struct Comment {
    /// 评论ID
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// 机厅 ID
    pub arcade_id: i32,
    /// 评论
    pub comment: String,
    /// 创建时间
    pub created_at: DateTime,
    /// 评分
    pub rating: Decimal128,
    /// 用户 ID
    pub user_id: ObjectId,
    /// 赞/踩数
    pub vote: i32,
}

/// 标签
#[derive(Serialize, Deserialize, ToResponse)]
pub struct Tag {
    /// 机厅 ID
    pub arcade_id: i32,
    /// 创建时间
    pub created_at: DateTime,
    /// 唯一 ID
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// 标签名
    pub name: String,
    /// 用户 ID
    pub user_id: ObjectId,
    /// 赞踩数
    pub vote: i32,
}
