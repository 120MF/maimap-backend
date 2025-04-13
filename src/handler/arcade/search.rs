use crate::db::get_mongodb_client;
use crate::env::DB_NAME;
use crate::handler::common::handle_error;
use crate::res::ApiResponse;
use crate::types::Arcade;
use anyhow::Result;
use mongodb::bson::Decimal128;
use mongodb::bson::doc;
use mongodb::{Collection, bson};
use salvo::prelude::*;
use serde::Deserialize;
use std::str::FromStr;
use tracing::info;

#[handler]
pub async fn search_arcades_handler(req: &mut Request, res: &mut Response) {
    match search_arcade(req).await {
        Ok(arcades) => res.render(Json(ApiResponse::success(arcades))),
        Err(e) => handle_error(res, e),
    }
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    name: Option<String>,
    lat: Option<f64>,
    lng: Option<f64>,
    range: Option<f64>, // 单位：米
    sort: Option<String>,
}

async fn search_arcade(req: &mut Request) -> Result<Vec<serde_json::Value>> {
    // 从请求中提取查询参数
    let query: SearchQuery = req.parse_queries::<SearchQuery>()?;

    // 创建查询条件
    let mut filter = doc! {};

    // 名称搜索
    if let Some(name) = query.name {
        info!(name);
        filter.insert("arcade_name", doc! { "$regex" : name });
    }

    // 地理位置搜索
    if let (Some(lat), Some(lng), Some(range)) = (query.lat, query.lng, query.range) {
        // 地球半径约为6371000米，将距离（米）转换为弧度
        let radius_in_radians = range / 6371000.0;

        filter.insert(
            "arcade_pos",
            doc! {
                "$geoWithin": {
                    "$centerSphere": [
                        [lng, lat],  // 坐标点作为数组
                        radius_in_radians  // 半径作为单独值
                    ]
                }
            },
        );
    }

    info!("{}", filter.to_string());

    // let mut sort_filter = doc! {};

    // 如果有排序参数
    // if let Some(sort_param) = query.sort {
    //     if let Some((field, order)) = sort_param.split_once(':') {
    //         let sort_order = if order == "desc" { -1 } else { 1 };
    //     }
    // }

    // 执行查询
    let client = get_mongodb_client();
    let coll_arcades: Collection<Arcade> = client.database(DB_NAME).collection("arcades");

    let mut find_operation = coll_arcades.find(filter);
    // 应用排序选项
    // if let Some(sort) = options.sort {
    //     find_operation = find_operation.sort(sort);
    // }

    let mut cursor = find_operation.await?;

    // 收集结果
    let mut results = Vec::new();
    while cursor.advance().await? {
        let arcade = cursor.deserialize_current()?;
        results.push(arcade.to_response());
    }

    Ok(results)
}
