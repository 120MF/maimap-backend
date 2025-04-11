use mongodb::Client;
use std::sync::OnceLock;

pub static MONGODB_CLIENT: OnceLock<Client> = OnceLock::new();

#[inline]
pub fn get_mongodb_client() -> &'static Client {
    MONGODB_CLIENT.get().unwrap()
}
