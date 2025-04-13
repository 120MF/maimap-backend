use maimap_backend::db::MONGODB_CLIENT;
use maimap_backend::env::{check_required_env_vars, database_uri};
use maimap_backend::handler::arcade::{get_arcade_by_id_handler, search_arcades_handler};

use mongodb::Client;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    check_required_env_vars();
    let client = Client::with_uri_str(database_uri())
        .await
        .expect("failed to connect");
    MONGODB_CLIENT.set(client).unwrap();

    let router = Router::with_path("arcades")
        .get(search_arcades_handler)
        .push(Router::with_path("{arcade_id}").get(get_arcade_by_id_handler));

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
