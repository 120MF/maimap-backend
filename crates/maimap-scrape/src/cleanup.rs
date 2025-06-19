use futures::stream::StreamExt;
use maimap_utils::db::{
    Arcade, Bson, Collection, Decimal128, Document, ObjectId, doc, ensure_mongodb_connected,
    get_mongodb_client,
};
use maimap_utils::errors::{AppError, Result};
use std::str::FromStr;
use tracing::info;

pub(crate) async fn convert_null_dead_to_bool() -> Result<u64> {
    ensure_mongodb_connected().await;

    info!("开始将 arcade_dead 为 null 的数据转换为 false...");

    let client = get_mongodb_client();
    let db = client.database("maimap");
    let collection: Collection<Document> = db.collection("arcades");

    // 筛选出 arcade_dead 字段为 null 的文档
    let filter = doc! { "arcade_dead": Bson::Null };

    // 定义更新操作，将 arcade_dead 设置为 false
    let update = doc! { "$set": { "arcade_dead": false } };

    // 执行批量更新
    let update_result = collection
        .update_many(filter, update)
        .await
        .map_err(AppError::Database)?;

    let total_updated = update_result.modified_count;

    info!(
        "arcade_dead 数据转换完成。总共更新了 {} 个文档。",
        total_updated
    );

    Ok(total_updated)
}
pub(crate) async fn convert_lat_lng_to_decimal128() -> Result<u64> {
    ensure_mongodb_connected().await;

    info!("开始将经纬度数据类型从 Double 或 String 转换为 Decimal128...");

    let client = get_mongodb_client();
    let db = client.database("maimap");
    let collection: Collection<Document> = db.collection("arcades");

    // 筛选出 arcade_lat 或 arcade_lng 字段为 Double 或 String 类型的文档
    let filter = doc! {
        "$or": [
            { "arcade_lat": { "$type": "double" } },
            { "arcade_lng": { "$type": "double" } },
            { "arcade_lat": { "$type": "string" } },
            { "arcade_lng": { "$type": "string" } }
        ]
    };

    let mut cursor = collection.find(filter).await.map_err(AppError::Database)?;

    let mut total_updated = 0;

    while let Some(result) = cursor.next().await {
        let doc = result.map_err(AppError::Database)?;
        let id = doc
            .get_object_id("_id")
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let mut updates = Document::new();

        // 检查并转换 arcade_lat 字段
        if let Some(lat_val) = doc.get("arcade_lat") {
            if let Bson::Double(lat) = lat_val {
                let lat_dec = Decimal128::from_str(&lat.to_string())
                    .map_err(|e| AppError::Parse(e.to_string()))?;
                updates.insert("arcade_lat", lat_dec);
            } else if let Bson::String(lat_str) = lat_val {
                if !lat_str.is_empty() {
                    let lat_dec = Decimal128::from_str(lat_str)
                        .map_err(|e| AppError::Parse(e.to_string()))?;
                    updates.insert("arcade_lat", lat_dec);
                }
            }
        }

        // 检查并转换 arcade_lng 字段
        if let Some(lng_val) = doc.get("arcade_lng") {
            if let Bson::Double(lng) = lng_val {
                let lng_dec = Decimal128::from_str(&lng.to_string())
                    .map_err(|e| AppError::Parse(e.to_string()))?;
                updates.insert("arcade_lng", lng_dec);
            } else if let Bson::String(lng_str) = lng_val {
                if !lng_str.is_empty() {
                    let lng_dec = Decimal128::from_str(lng_str)
                        .map_err(|e| AppError::Parse(e.to_string()))?;
                    updates.insert("arcade_lng", lng_dec);
                }
            }
        }

        if !updates.is_empty() {
            let update_doc = doc! { "$set": updates };
            let update_result = collection
                .update_one(doc! { "_id": id }, update_doc)
                .await
                .map_err(AppError::Database)?;

            if update_result.modified_count > 0 {
                total_updated += update_result.modified_count;
                info!("已更新文档 (ID: {}) 的经纬度类型", id);
            }
        }
    }

    info!("经纬度类型转换完成。总共更新了 {} 个文档。", total_updated);

    Ok(total_updated)
}

pub(crate) async fn remove_duplicate_arcades() -> Result<u64> {
    // 确保MongoDB已连接
    ensure_mongodb_connected().await;

    info!("开始清理重复的机厅记录...");

    let client = get_mongodb_client();
    let db = client.database("maimap");
    let collection = db.collection::<Arcade>("arcades");

    // 使用聚合管道查找具有相同arcade_id和arcade_name的记录
    let pipeline = vec![
        doc! {
            "$group": {
                "_id": {
                    "arcade_id": "$arcade_id",
                    "arcade_name": "$arcade_name"
                },
                "ids": { "$push": "$_id" },
                "count": { "$sum": 1 }
            }
        },
        doc! {
            "$match": {
                "count": { "$gt": 1 }
            }
        },
    ];

    let mut cursor = collection.aggregate(pipeline).await?;
    let mut total_deleted = 0;

    while let Some(result) = cursor.next().await {
        let group = result.map_err(AppError::Database)?;

        let ids = match group.get_array("ids") {
            Ok(ids) if ids.len() > 1 => ids,
            _ => continue,
        };

        // 获取arcade_id和arcade_name用于日志记录
        let id_doc = group
            .get_document("_id")
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let arcade_id = id_doc
            .get_i32("arcade_id")
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let arcade_name = id_doc
            .get_str("arcade_name")
            .map_err(|e| AppError::Parse(e.to_string()))?;

        // 保留第一个文档，删除其余的
        let mut ids_to_delete = Vec::new();
        for (i, id_bson) in ids.iter().enumerate() {
            if i > 0 {
                // 跳过第一个ID
                if let ObjectId(oid) = id_bson {
                    ids_to_delete.push(oid);
                }
            }
        }

        if !ids_to_delete.is_empty() {
            let delete_result = collection
                .delete_many(doc! { "_id": { "$in": ids_to_delete } })
                .await
                .map_err(AppError::Database)?;

            let deleted_count = delete_result.deleted_count;
            total_deleted += deleted_count;

            info!(
                "删除了 {} 个重复的机厅记录: arcade_id={}, arcade_name={}",
                deleted_count, arcade_id, arcade_name
            );
        }
    }

    info!("总共删除了 {} 个重复的机厅记录", total_deleted);

    Ok(total_deleted)
}
