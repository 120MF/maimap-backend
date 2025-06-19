use maimap_utils::env::qmap_key;
use maimap_utils::errors::AppError;
use maimap_utils::types::Point;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use tracing::info;

#[derive(Deserialize)]
pub(crate) struct GeocoderResponse {
    message: String,
    status: i32,
    result: Option<GeocoderResult>,
}

#[derive(Deserialize)]
pub(crate) struct GeocoderResult {
    location: GeoLocation,
}

#[derive(Deserialize, Clone)]
pub(crate) struct GeoLocation {
    pub lat: f64,
    pub lng: f64,
}

impl GeoLocation {
    pub(crate) fn to_point(&self) -> Point {
        Point::new(self.lng, self.lat)
    }
}
impl Default for GeoLocation {
    fn default() -> Self {
        Self { lat: 0.0, lng: 0.0 }
    }
}

impl Display for GeoLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.lat, self.lng)
    }
}
pub(crate) async fn get_geo_location(address: &str) -> maimap_utils::errors::Result<GeoLocation> {
    let client = reqwest::Client::new();
    let max_retries = 3;
    let base_delay = Duration::from_secs(2);
    let mut use_policy = false;

    for attempt in 0..=max_retries {
        let key = qmap_key();
        let mut params = vec![("address", address), ("key", &key)];

        // 如果需要添加policy参数
        if use_policy {
            params.push(("policy", "1"));
        }

        let response = client
            .get("https://apis.map.qq.com/ws/geocoder/v1/")
            .query(&params)
            .send()
            .await?;

        let geocoder_response: GeocoderResponse = response.json().await?;

        if geocoder_response.status == 0 {
            if let Some(result) = geocoder_response.result {
                return Ok(result.location);
            }
        } else if geocoder_response.status == 348 && !use_policy {
            use_policy = true;
            info!("收到状态码348，添加policy=1参数并立即重试");
            continue;
        } else if attempt < max_retries {
            let delay = base_delay.mul_f32(1.5_f32.powi(attempt));
            info!(
                "地址解析失败，状态码: {}，消息: {}，将在 {:?} 后重试 ({}/{})",
                geocoder_response.status,
                geocoder_response.message,
                delay,
                attempt + 1,
                max_retries
            );

            tokio::time::sleep(delay).await;
            continue;
        }

        return Err(AppError::Geocoder(geocoder_response.message).into());
    }

    unreachable!("重试循环应该返回结果或错误");
}
