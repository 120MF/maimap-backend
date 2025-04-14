use maimap_backend::db::MONGODB_CLIENT;
use maimap_backend::env::{check_required_env_vars, database_uri};
use maimap_backend::router::router;

use mongodb::Client;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    check_required_env_vars();
    tracing_subscriber::fmt().init();
    let client = Client::with_uri_str(database_uri())
        .await
        .expect("failed to connect");
    MONGODB_CLIENT.set(client).unwrap();

    let router = router();

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
