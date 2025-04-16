use crate::res::ApiResponse;
use maimap_utils::db::Collection;
use maimap_utils::db::Int32;
use maimap_utils::db::doc;
use maimap_utils::db::get_mongodb_client;
use maimap_utils::env::DB_NAME;
use maimap_utils::errors::AppError;
use maimap_utils::errors::Result;
use maimap_utils::types::Arcade;
use salvo::prelude::*;

use crate::handler::common::handle_error;

#[handler]
pub async fn get_arcade_by_id_handler(req: &mut Request, res: &mut Response) {
    match get_arcade_by_id(req).await {
        Ok(arcade) => res.render(Json(ApiResponse::success(arcade))),
        Err(e) => handle_error(res, e),
    }
}

async fn get_arcade_by_id(req: &mut Request) -> Result<serde_json::Value> {
    let arcade_id = req
        .param::<i32>("arcade_id")
        .ok_or_else(|| AppError::Validation("缺少arcade_id参数".to_string()))?;
    let client = get_mongodb_client();
    let coll_arcades: Collection<Arcade> = client.database(DB_NAME).collection("arcades");

    let result = coll_arcades
        .find_one(doc! {"arcade_id": Int32(arcade_id)})
        .await?;

    Ok(match result {
        Some(arcade) => arcade.to_response(),
        None => serde_json::json!({}),
    })
}
