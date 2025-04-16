use maimap_server::router::router;
use maimap_utils::db::ensure_mongodb_connected;
use maimap_utils::env::check_required_env_vars;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    ensure_mongodb_connected().await;
    check_required_env_vars();
    tracing_subscriber::fmt().init();

    let router = router();

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
