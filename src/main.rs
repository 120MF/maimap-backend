mod backup;
use backup::backup_database;

mod env;
use env::{DB_NAME, backup_path, database_uri};

mod db;
use db::{MONGODB_CLIENT, get_mongodb_client};
mod res;
use res::ApiResponse;

mod scrape;
mod types;

use types::Arcade;

use mongodb::{Client, Collection, bson::Bson::Int32, bson::doc};
use salvo::prelude::*;

use crate::scrape::{scheduled_scrape, scrape_arcades};
use thiserror::Error;
use tracing::{error, info};

// Custom error type for MongoDB operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("MongoDB错误：{0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("参数错误：{0}")]
    Param(String),
}

#[handler]
async fn get_arcades_by_id(req: &mut Request, res: &mut Response) {
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let client = Client::with_uri_str(database_uri())
        .await
        .expect("failed to connect");
    MONGODB_CLIENT.set(client).unwrap();

    match backup_database() {
        Ok(_) => info!("数据库备份成功：{}maimap.gz", backup_path()),
        Err(e) => error!("数据库备份失败：{}", e),
    }

    tokio::spawn(scheduled_scrape());

    let router =
        Router::with_path("arcades").push(Router::with_path("{arcade_id}").get(get_arcades_by_id));

    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
