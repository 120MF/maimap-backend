use maimap_backend::backup::backup_database;
use maimap_backend::db::MONGODB_CLIENT;
use maimap_backend::env::{backup_path, database_uri};
use maimap_backend::handler::get_arcades_by_id;

use mongodb::Client;
use salvo::prelude::*;
use tracing::{error, info};

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

    let router =
        Router::with_path("arcades").push(Router::with_path("{arcade_id}").get(get_arcades_by_id));

    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
