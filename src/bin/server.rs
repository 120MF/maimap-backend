use maimap_backend::db::MONGODB_CLIENT;
use maimap_backend::env::database_uri;
use maimap_backend::maimap_handler::get_arcades_by_id;

use mongodb::Client;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let client = Client::with_uri_str(database_uri())
        .await
        .expect("failed to connect");
    MONGODB_CLIENT.set(client).unwrap();

    let router =
        Router::with_path("arcades").push(Router::with_path("{arcade_id}").get(get_arcades_by_id));

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
