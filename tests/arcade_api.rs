mod env;
mod types;

#[cfg(test)]
mod tests {
    use crate::env::{check_required_env_vars, database_uri};
    use crate::types::{ApiResponse, Arcade, Comment};
    use maimap_backend::db::MONGODB_CLIENT;
    use maimap_backend::router::router;
    use mongodb::Client;
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};
    use std::time::Duration;

    async fn ensure_mongodb_connected() {
        if MONGODB_CLIENT.get().is_none() {
            let client = Client::with_uri_str(database_uri())
                .await
                .expect("failed to connect");

            // 忽略可能的错误
            let _ = MONGODB_CLIENT.set(client);
        }
    }

    #[tokio::test]
    async fn test_get_arcade_by_id() {
        check_required_env_vars();
        ensure_mongodb_connected().await;
        let service = Service::new(router());
        let content: ApiResponse<Arcade> =
            TestClient::get("http://127.0.0.1:5800/arcades/1514".to_string())
                .send(&service)
                .await
                .take_json()
                .await
                .expect("解析JSON失败");
        assert_eq!(content.success, true);
        assert_eq!(content.data.arcade_id, 1514);
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }

    #[tokio::test]
    async fn test_search_arcade() {
        check_required_env_vars();
        ensure_mongodb_connected().await;
        let service = Service::new(router());
        let content: ApiResponse<Vec<Arcade>> =
            TestClient::get("http://127.0.0.1:5800/arcades?name=环游嘉年华&lat=39.909333&lng=116.397183&range=1000000&sort=Distance&page_index=1&page_size=20".to_string())
                .send(&service)
                .await
                .take_json()
                .await
                .expect("解析JSON失败");
        content.count.unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    #[tokio::test]
    async fn test_get_comments() {
        check_required_env_vars();
        ensure_mongodb_connected().await;
        let service = Service::new(router());
        let content: ApiResponse<Vec<Comment>> =
            TestClient::get("http://127.0.0.1:5800/arcades/1514/comments".to_string())
                .send(&service)
                .await
                .take_json()
                .await
                .expect("解析JSON失败");
        content.count.unwrap();
        assert_eq!(content.success, true);
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
