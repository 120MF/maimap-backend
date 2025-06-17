use crate::env::{DB_NAME, database_uri, test_database_uri};

use anyhow::Result;

pub use crate::types::Arcade;
pub use mongodb::bson::Bson::Int32;
pub use mongodb::bson::Bson::ObjectId;
pub use mongodb::bson::DateTime;
pub use mongodb::bson::Decimal128;
pub use mongodb::bson::Document;
pub use mongodb::bson::doc;
pub use mongodb::bson::to_bson;
pub use mongodb::options::Collation;
pub use mongodb::{Client, Collection};
use std::sync::OnceLock;

pub static MONGODB_CLIENT: OnceLock<Client> = OnceLock::new();

pub async fn ensure_mongodb_connected() {
    if MONGODB_CLIENT.get().is_none() {
        let client = Client::with_uri_str(database_uri())
            .await
            .expect("无法连接到数据库");
        let _ = MONGODB_CLIENT.set(client);
    }
}

pub async fn ensure_test_mongodb_connected() {
    if MONGODB_CLIENT.get().is_none() {
        let client = Client::with_uri_str(test_database_uri())
            .await
            .expect("无法连接到测试数据库");
        let _ = MONGODB_CLIENT.set(client);
    }
}

#[inline]
pub fn get_mongodb_client() -> &'static Client {
    MONGODB_CLIENT.get().unwrap()
}

pub async fn get_max_arcade_id() -> Result<i32> {
    let client = get_mongodb_client();
    let collection: Collection<Arcade> = client.database(DB_NAME).collection("arcades");
    let options = mongodb::options::FindOneOptions::builder()
        .sort(doc! {"arcade_id": -1})
        .build();

    match collection.find_one(doc! {}).with_options(options).await {
        Ok(None) => Ok(0),
        Ok(Some(arcade)) => Ok(arcade.arcade_id),
        Err(e) => Err(e.into()),
    }
}

pub async fn insert_many_arcades(arcades: Vec<Arcade>) -> Result<(), mongodb::error::Error> {
    if arcades.is_empty() {
        return Ok(());
    }

    let client = get_mongodb_client();
    let collection: Collection<Arcade> = client.database(DB_NAME).collection("arcades");
    collection.insert_many(arcades).await?;
    Ok(())
}
