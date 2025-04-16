use headless_chrome::{Browser, LaunchOptions, Tab};

use maimap_utils::backup::backup_database;
use maimap_utils::db::{
    DateTime, Decimal128, ensure_mongodb_connected, get_max_arcade_id, insert_many_arcades,
};
use maimap_utils::env::{check_required_env_vars, qmap_key};
use maimap_utils::errors::{AppError, Context, Result};
use maimap_utils::types::{Arcade, Point};

use scraper::{Html, Selector};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;
use tracing::{error, info};

#[derive(Deserialize)]
struct GeocoderResponse {
    message: String,
    status: i32,
    result: Option<GeocoderResult>,
}

#[derive(Deserialize)]
struct GeocoderResult {
    location: GeoLocation,
}

#[derive(Deserialize, Clone)]
struct GeoLocation {
    lat: f64,
    lng: f64,
}

impl GeoLocation {
    fn to_point(&self) -> Point {
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
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    info!("执行定时爬取华立机厅任务");
    check_required_env_vars();
    ensure_mongodb_connected().await;

    match scrape_arcades().await {
        Ok(_) => {
            info!("爬取任务成功！");
            match backup_database().await {
                Ok(_) => info!("备份数据库成功！"),
                Err(e) => error!("备份数据库失败！{}", e),
            }
        }
        Err(e) => error!("爬取任务失败！{}", e),
    }
}

pub async fn scrape_arcades() -> Result<()> {
    info!("开始爬取华立官网机厅");
    let content;
    {
        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(true)
                .sandbox(false)
                .build()?,
        )
        .context("运行无头Chrome失败")?;
        let tab = browser.new_tab().context("新建浏览器Tab失败")?;
        tab.navigate_to("http://wc.wahlap.net/maidx/location/index.html")
            .context("导航到指定页面失败")?;
        tab.wait_until_navigated().context("等待指定页面加载失败")?;
        wait_for_store_list(&tab).context("等待store_list加载失败")?;
        content = tab.get_content().context("获取页面content失败")?;
    }
    match parse_store_list(&content).await {
        Ok(arcades) => match insert_many_arcades(arcades).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e),
    }
}

fn wait_for_store_list(tab: &Tab) -> Result<()> {
    tab.wait_for_element(".store_list")?;
    tab.reload(false, None)?;
    tab.wait_for_element(".store_list")?;
    std::thread::sleep(Duration::from_secs(5));

    Ok(())
}

async fn get_geo_location(address: &str) -> Result<GeoLocation> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://apis.map.qq.com/ws/geocoder/v1/")
        .query(&[("address", address), ("key", &qmap_key())])
        .send()
        .await?;
    let geocoder_response: GeocoderResponse = response.json().await?;
    if geocoder_response.status == 0 {
        if let Some(result) = geocoder_response.result {
            return Ok(result.location);
        }
    }
    Err(AppError::Geocoder(geocoder_response.message).into())
}
async fn parse_store_list(html: &str) -> Result<Vec<Arcade>> {
    let time = DateTime::now();

    let document = Html::parse_document(html);
    let mut arcades = Vec::new();
    let ul_selector = Selector::parse(".store_list").map_err(|e| AppError::Parse(e.to_string()))?;
    let li_selector = Selector::parse("li").map_err(|e| AppError::Parse(e.to_string()))?;

    let store_name_selector =
        Selector::parse("span.store_name").map_err(|e| AppError::Parse(e.to_string()))?;
    let store_address_selector =
        Selector::parse("span.store_address").map_err(|e| AppError::Parse(e.to_string()))?;
    let max_id = get_max_arcade_id().await?;
    info!("max_id: {}", max_id);

    let mut id_counter: i32 = 0;
    for (_, store_list) in document.select(&ul_selector).enumerate() {
        for li in store_list.select(&li_selector) {
            id_counter += 1;
            if id_counter <= max_id {
                continue; // 只从已有机厅id最大处开始爬取
            }
            let store_name = li
                .select(&store_name_selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .ok_or_else(|| AppError::Parse("store_name".to_string()))?;

            let store_address = li
                .select(&store_address_selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .ok_or_else(|| AppError::Parse("store_address".to_string()))?;

            let location = get_geo_location(&store_address).await?;
            info!(
                "\n爬取到新机厅：\n{}\n{}\n{}\n{}\n\n",
                id_counter, store_name, store_address, location,
            );

            let arcade = Arcade {
                arcade_id: id_counter,
                arcade_name: store_name,
                arcade_address: store_address,
                arcade_dead: false,
                arcade_cost: None,
                arcade_count: None,
                arcade_lat: Decimal128::from_str(&location.lat.to_string())?,
                arcade_lng: Decimal128::from_str(&location.lng.to_string())?,
                arcade_pos: Some(location.to_point()),
                created_at: time,
            };
            arcades.push(arcade);
            tokio::time::sleep(Duration::from_millis(700)).await; //腾讯地图免费API并发调用限制 3次/s
        }
    }
    Ok(arcades)
}
