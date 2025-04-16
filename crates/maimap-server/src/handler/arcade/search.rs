use crate::handler::common::{handle_error, paginate_results};
use crate::res::ApiResponse;
use maimap_utils::db::{Collation, Collection, Document, doc, get_mongodb_client, to_bson};
use maimap_utils::env::DB_NAME;
use maimap_utils::errors::AppError;
use maimap_utils::errors::Result;
use maimap_utils::types::Arcade;
use salvo::prelude::*;
use serde::Deserialize;

#[handler]
pub async fn search_arcades_handler(req: &mut Request, res: &mut Response) {
    match search_arcade(req).await {
        Ok((arcades, count)) => res.render(Json(ApiResponse::success(arcades).with_count(count))),
        Err(e) => handle_error(res, e),
    }
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    name: Option<String>,
    lat: Option<f64>,
    lng: Option<f64>,
    range: Option<f64>, // 单位：米
    page_index: Option<u32>,
    page_size: Option<u32>,
    sort: Option<String>,
}

async fn search_arcade(req: &mut Request) -> Result<(Vec<serde_json::Value>, usize)> {
    // 从请求中提取查询参数
    let query: SearchQuery = req.parse_queries::<SearchQuery>()?;

    // 创建查询条件
    let mut pipeline: Vec<Document> = Vec::new();

    //地理位置搜索
    match generate_geo_doc(&query)? {
        None => {}
        Some(document) => pipeline.push(document),
    }

    //名称搜索
    if let Some(name) = &query.name {
        pipeline.push(doc! {"$match": {"arcade_name": {"$regex": name}}})
    }

    //构建排序
    let collation = Collation::builder().locale("zh").build();
    let sort_doc = match query.sort.as_deref() {
        Some("Distance") => doc! {"$sort": {"distance": 1}},
        Some("Pinyin") => doc! {"$sort": {"arcade_name": 1}},
        _ => doc! {"$sort": {"arcade_id": 1}},
    };
    pipeline.push(sort_doc);

    // 修改字段以符合API要求
    serialize_data(&mut pipeline);

    // 执行查询
    let client = get_mongodb_client();
    let coll_arcades: Collection<Arcade> = client.database(DB_NAME).collection("arcades");
    let mut cursor = coll_arcades
        .aggregate(pipeline)
        .collation(collation)
        .await?;

    // 收集结果
    let mut results = Vec::new();
    while cursor.advance().await? {
        let doc = cursor.deserialize_current()?;
        let json_value: serde_json::Value = to_bson(&doc)
            .map_err(|e| AppError::Serialize(e.to_string()))?
            .into();
        results.push(json_value);
    }
    let total_count = results.len();
    let paged_results = paginate_results(&results, query.page_index, query.page_size)?;

    Ok((paged_results, total_count))
}

fn generate_geo_doc(query: &SearchQuery) -> Result<Option<Document>> {
    if let (Some(lat), Some(lng), Some(range)) = (query.lat, query.lng, query.range) {
        Ok(Some(doc! {
            "$geoNear": {
                "near" : {
                    "type" : "Point",
                    "coordinates" : [lng, lat]
                },
                "distanceField": "distance",
                "spherical" : true,
                "maxDistance": range
            }
        }))
    } else if query.lat.is_some() != query.lng.is_some()
        || query.lat.is_some() != query.range.is_some()
    {
        return Err(AppError::Validation(
            "地理位置搜索需要同时提供lat、lng和range三个参数".to_string(),
        )
        .into());
    } else {
        Ok(None)
    }
}

fn serialize_data(pipeline: &mut Vec<Document>) {
    pipeline.push(doc! {
        "$project": {
            "_id": 0,
            "arcade_pos": 0
        },
    });

    pipeline.push(doc! {
        "$addFields": {
            "arcade_lat": { "$toDouble": "$arcade_lat" },
            "arcade_lng": { "$toDouble": "$arcade_lng" },
            "created_at": { "$dateToString": {
                "format": "%Y-%m-%dT%H:%M:%S.%LZ",
                "date": "$created_at"
            }}
        },
    });
}
