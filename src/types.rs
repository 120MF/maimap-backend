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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

    pub arcade_pos: Option<Point>,
    /// 机厅名
    pub arcade_name: String,
    /// 创建时间
    pub created_at: DateTime,
}

impl Arcade {
    pub fn to_response(&self) -> serde_json::Value {
        serde_json::json!({
            "arcade_id": self.arcade_id,
            "arcade_name": self.arcade_name,
            "arcade_address": self.arcade_address,
            "arcade_lat": self.arcade_lat.to_string().parse::<f64>().unwrap_or(0.0),
            "arcade_lng": self.arcade_lng.to_string().parse::<f64>().unwrap_or(0.0),
            "arcade_dead": self.arcade_dead,
            "created_at": self.created_at.try_to_rfc3339_string().unwrap_or_default(),
            "arcade_count": self.arcade_count,
            "arcade_cost": self.arcade_cost,
        })
    }
}

pub enum SortMethod {}
