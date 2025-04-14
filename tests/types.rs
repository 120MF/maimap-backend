use serde::{Deserialize, Serialize};

#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub count: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arcade {
    /// 机厅地址
    pub arcade_address: String,
    /// 单局花销
    pub arcade_cost: Option<f64>,
    /// 机台数量
    pub arcade_count: Option<i64>,
    /// 机厅存活情况
    pub arcade_dead: bool,
    /// 机厅ID
    pub arcade_id: i64,
    /// 机厅纬度
    pub arcade_lat: f64,
    /// 机厅经度
    pub arcade_lng: f64,
    /// 机厅名
    pub arcade_name: String,
    /// 创建时间
    pub created_at: String,
}
