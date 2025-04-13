use crate::db::get_mongodb_client;
use crate::env::DB_NAME;
use crate::res::ApiResponse;
use crate::types::Arcade;
use mongodb::Collection;
use mongodb::bson::Bson::Int32;
use mongodb::bson::doc;
use salvo::prelude::*;
use salvo::{Request, Response, handler};

#[handler]
pub async fn get_arcades_by_id(req: &mut Request, res: &mut Response) {
    let arcade_id = match req.param::<i32>("arcade_id") {
        Some(id) => Int32(id),
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ApiResponse::<()>::error("缺少arcade_id参数")));
            return;
        }
    };
    let client = get_mongodb_client();
    let coll_arcades: Collection<Arcade> = client.database(DB_NAME).collection("arcades");
    match coll_arcades.find_one(doc! { "arcade_id": arcade_id }).await {
        Ok(Some(arcade)) => res.render(Json(ApiResponse::success(arcade.to_response()))),
        Ok(None) => res.render(Json(ApiResponse::success(serde_json::json!({})))),
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(ApiResponse::<()>::error(format!(
                "数据库错误：{:?}",
                e
            ))))
        }
    }
}
