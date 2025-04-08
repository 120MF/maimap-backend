mod types;
use types::Arcade;

use mongodb::{
    Client, Collection, IndexModel, bson::Bson::Int32, bson::Document, bson::doc,
    bson::oid::ObjectId, options::IndexOptions,
};
use salvo::{oapi::extract::JsonBody, prelude::*};
use std::sync::OnceLock;

use thiserror::Error;

const DB_NAME: &str = "maimap";

// Custom error type for MongoDB operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("MongoDB Error")]
    ErrorMongo(#[from] mongodb::error::Error),
}
static MONGODB_CLIENT: OnceLock<Client> = OnceLock::new();
#[inline]
pub fn get_mongodb_client() -> &'static Client {
    MONGODB_CLIENT.get().unwrap()
}

#[handler]
async fn get_arcades_by_id(req: &mut Request, res: &mut Response) {
    let arcade_id = Int32(req.param::<i32>("arcade_id").unwrap_or_default());
    let client = get_mongodb_client();
    let coll_arcades: Collection<Arcade> = client.database(DB_NAME).collection("arcades");
    match coll_arcades.find_one(doc! { "arcade_id": arcade_id }).await {
        Ok(Some(arcade)) => res.render(Json(serde_json::json!(
            {
              "arcade_id": arcade.arcade_id,
              "arcade_name": arcade.arcade_name,
              "arcade_address": arcade.arcade_address,
              "arcade_lat": arcade.arcade_lat.to_string().parse::<f64>().unwrap_or(0.0),
              "arcade_lng": arcade.arcade_lng.to_string().parse::<f64>().unwrap_or(0.0),
              "arcade_dead": arcade.arcade_dead,
              "created_at": arcade.created_at.try_to_rfc3339_string().unwrap_or("".parse().unwrap()),
              "arcade_count": arcade.arcade_count,
              "arcade_cost": arcade.arcade_cost,
            }
        ))),
        Ok(None) => res.render(Json({})),
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": format!("{:?}",e)
            })))
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let mongodb_uri = "mongodb://localhost:27017/";
    let client = Client::with_uri_str(mongodb_uri)
        .await
        .expect("failed to connect");
    MONGODB_CLIENT.set(client).unwrap();

    let router =
        Router::with_path("arcades").push(Router::with_path("{arcade_id}").get(get_arcades_by_id));

    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
