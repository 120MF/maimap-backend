use crate::env::DB_NAME;
use crate::types::Arcade;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use std::sync::OnceLock;

pub static MONGODB_CLIENT: OnceLock<Client> = OnceLock::new();

#[inline]
pub fn get_mongodb_client() -> &'static Client {
    MONGODB_CLIENT.get().unwrap()
}

pub async fn get_max_arcade_id() -> i32 {
    let client = get_mongodb_client();
    let collection: Collection<Arcade> = client.database(DB_NAME).collection("arcades");

    let options = mongodb::options::FindOneOptions::builder()
        .sort(doc! {"arcade_id": -1})
        .build();

    match collection.find_one(doc! {}).with_options(options).await {
        Ok(Some(arcade)) => arcade.arcade_id as i32,
        _ => 0,
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
