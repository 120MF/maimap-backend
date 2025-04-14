use dotenv::dotenv;
use std::env;
use tracing::warn;

pub fn check_required_env_vars() {
    dotenv().ok();
    let required_vars = ["TEST_DATABASE_URI"];

    for var in required_vars {
        if let Err(_) = env::var(var) {
            warn!("缺少环境变量：{}", var);
        }
    }
}

pub fn database_uri() -> String {
    env::var("TEST_DATABASE_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
}
