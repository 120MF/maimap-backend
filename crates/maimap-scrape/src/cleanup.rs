use futures::stream::StreamExt;
use maimap_utils::db::{Arcade, ObjectId, doc, ensure_mongodb_connected, get_mongodb_client};
use maimap_utils::errors::{AppError, Result};
use tracing::info;

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
        let group = result.map_err(|e| AppError::Database(e))?;

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
                .map_err(|e| AppError::Database(e))?;

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
