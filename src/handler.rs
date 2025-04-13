use crate::db::get_mongodb_client;
use crate::env::DB_NAME;

use crate::errors::AppError;
use anyhow::Result;

use crate::res::ApiResponse;
use crate::types::Arcade;
use mongodb::Collection;
use mongodb::bson::Bson::Int32;
use mongodb::bson::doc;
use salvo::prelude::*;
use salvo::{Request, Response, handler};

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

fn handle_error(res: &mut Response, err: anyhow::Error) {
    if let Some(app_err) = err.downcast_ref::<AppError>() {
        match app_err {
            AppError::Validation(_) => res.status_code(StatusCode::BAD_REQUEST),
            _ => res.status_code(StatusCode::INTERNAL_SERVER_ERROR),
        };
    } else {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }

    res.render(Json(ApiResponse::<()>::error(err.to_string())));
}
