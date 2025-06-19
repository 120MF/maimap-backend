use crate::env::{DB_NAME, database_uri, test_database_uri};
use futures_util::stream::StreamExt;

use anyhow::Result;

pub use crate::types::Arcade;
pub use mongodb::bson::Bson;
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

pub async fn get_all_arcades() -> Result<Vec<Arcade>> {
    let client = get_mongodb_client();
    let collection: Collection<Arcade> = client.database(DB_NAME).collection("arcades");

    let find_options = mongodb::options::FindOptions::builder()
        .sort(doc! { "arcade_id": 1 })
        .build();

    let mut cursor = collection.find(doc! {}).with_options(find_options).await?;
    let mut arcades = Vec::new();
    while let Some(result) = cursor.next().await {
        let arcade = result?;
        arcades.push(arcade);
    }

    Ok(arcades)
}

pub async fn update_arcade(arcade: &Arcade) -> Result<()> {
    let client = get_mongodb_client();
    let collection: Collection<Arcade> = client.database(DB_NAME).collection("arcades");

    let filter = doc! { "arcade_id": arcade.arcade_id };

    // 使用整个文档进行替换，保留 _id 字段
    let result = collection.replace_one(filter, arcade).await?;

    if result.matched_count == 0 {
        return Err(anyhow::anyhow!(
            "未找到要更新的机厅：ID {}",
            arcade.arcade_id
        ));
    }

    Ok(())
}
