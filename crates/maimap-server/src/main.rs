use maimap_server::router::router;
use maimap_utils::db::ensure_mongodb_connected;
use maimap_utils::env::check_required_env_vars;
use salvo::cors::{Cors, AllowHeaders, AllowOrigin, AllowMethods};
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    check_required_env_vars();
    ensure_mongodb_connected().await;
    tracing_subscriber::fmt().init();

    let router = router();
    let cors = Cors::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .into_handler();

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    let service = Service::new(router).hoop(cors);
    Server::new(acceptor).serve(service).await;
}
