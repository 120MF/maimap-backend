use crate::handler::common::handle_error;
use crate::res::ApiResponse;
use maimap_utils::db::Collection;
use maimap_utils::db::doc;
use maimap_utils::db::get_mongodb_client;
use maimap_utils::env::DB_NAME;
use maimap_utils::errors::AppError;
use maimap_utils::errors::Result;
use maimap_utils::traits::ToResponse;
use maimap_utils::types::Comment;
use salvo::prelude::Json;
use salvo::{Request, Response, handler};

#[handler]
pub async fn get_comments_handler(req: &mut Request, res: &mut Response) {
    match get_comment(req).await {
        Ok((comments, count)) => res.render(Json(ApiResponse::success(comments).with_count(count))),
        Err(e) => handle_error(res, e),
    }
}

async fn get_comment(req: &mut Request) -> Result<(Vec<serde_json::Value>, usize)> {
    let arcade_id = req
        .param::<i32>("arcade_id")
        .ok_or_else(|| AppError::Validation("缺少arcade_id参数".to_string()))?;

    let client = get_mongodb_client();

    let coll_comments: Collection<Comment> = client.database(DB_NAME).collection("comments");

    let mut cursor = coll_comments.find(doc! {"arcade_id": arcade_id}).await?;

    let mut comments = Vec::new();
    while cursor.advance().await? {
        let doc = cursor.deserialize_current()?;
        comments.push(doc.to_response());
    }
    let count = comments.len();
    Ok((comments, count))
}
