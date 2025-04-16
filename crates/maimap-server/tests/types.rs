use serde::{Deserialize, Serialize};

#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
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

/// 评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// 机厅 ID
    pub arcade_id: i64,
    /// 评论
    pub comment: String,
    /// 创建时间
    pub created_at: String,
    /// 评论ID
    pub id: String,
    /// 评分
    pub rating: f64,
    /// 用户 ID
    pub user_id: String,
    /// 赞/踩数
    pub vote: i64,
}

/// 标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// 机厅 ID
    pub arcade_id: i64,
    /// 创建时间
    pub created_at: String,
    /// 唯一 ID
    pub id: String,
    /// 标签名
    pub name: String,
    /// 用户 ID
    pub user_id: String,
    /// 赞踩数
    pub vote: i64,
}
