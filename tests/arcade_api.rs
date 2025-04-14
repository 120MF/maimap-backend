mod env;
mod types;

#[cfg(test)]
mod tests {
    use crate::env::{check_required_env_vars, database_uri};
    use crate::types::{ApiResponse, Arcade};
    use maimap_backend::db::MONGODB_CLIENT;
    use maimap_backend::router::router;
    use mongodb::Client;
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};

    #[tokio::test]

    async fn test_get_arcade_by_id() {
        tracing_subscriber::fmt().init();
        check_required_env_vars();
        let client = Client::with_uri_str(database_uri())
            .await
            .expect("failed to connect");
        MONGODB_CLIENT.set(client).unwrap();
        let service = Service::new(router());
        let content: ApiResponse<Arcade> =
            TestClient::get("http://127.0.0.1:5800/arcades/1514".to_string())
                .send(&service)
                .await
                .take_json()
                .await
                .expect("解析JSON失败");

        assert_eq!(content.data.arcade_id, 1514);
    }
}
