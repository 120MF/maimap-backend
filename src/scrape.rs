use crate::db::get_max_arcade_id;
use headless_chrome::{Browser, LaunchOptions, Tab};

use crate::types::{Arcade, Point};
use mongodb::bson::{DateTime, Decimal128};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Deserialize)]
struct GeocoderResponse {
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
        Point::new(self.lat, self.lng)
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

fn wait_for_store_list(tab: &Tab) -> Result<(), Box<dyn Error>> {
    tab.wait_for_element(".store_list")?;
    tab.reload(false, None)?;
    tab.wait_for_element(".store_list")?;
    std::thread::sleep(Duration::from_secs(5));

    Ok(())
}

async fn get_geo_location(address: &str) -> Result<Option<GeoLocation>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://apis.map.qq.com/ws/geocoder/v1/")
        .query(&[
            ("address", address),
            ("key", "4BQBZ-6DJWA-MJDKJ-CEHME-I4AL7-IDBK7"),
        ])
        .send()
        .await?;
    let geocoder_response: GeocoderResponse = response.json().await?;
    if geocoder_response.status == 0 {
        if let Some(result) = geocoder_response.result {
            return Ok(Some(result.location));
        }
    }
    Ok(None)
}
async fn parse_store_list(html: &str) -> Result<Vec<Arcade>, Box<dyn Error>> {
    let time = DateTime::now();

    let document = Html::parse_document(html);
    let mut arcades = Vec::new();
    let ul_selector = Selector::parse(".store_list").unwrap();
    let li_selector = Selector::parse("li").unwrap();

    let store_name_selector = Selector::parse("span.store_name")?;
    let store_address_selector = Selector::parse("span.store_address")?;
    let max_id = get_max_arcade_id().await;
    info!("max_id: {}", max_id);

    let mut id_counter: i32 = 0;
    for (_, store_list) in document.select(&ul_selector).enumerate() {
        for li in store_list.select(&li_selector) {
            id_counter += 1;
            if id_counter < max_id {
                continue;
            }
            let store_name = li
                .select(&store_name_selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| "未知店名".to_string());

            let store_address = li
                .select(&store_address_selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| "未知地址".to_string());

            let location;
            match get_geo_location(&store_address).await {
                Ok(loc) => location = loc.unwrap_or_default(),
                Err(e) => {
                    error!(
                        "获取腾讯地图API失败，为防止数据污染，放弃本次更新。 Error: {}",
                        e
                    );
                    return Err(e);
                }
            }
            info!(
                "{}\n{}\n{}\n{}\n\n",
                id_counter, store_name, store_address, location,
            );

            let arcade = Arcade {
                arcade_id: id_counter as i32,
                arcade_name: store_name,
                arcade_address: store_address,
                arcade_dead: false,
                arcade_cost: None,
                arcade_count: None,
                arcade_lat: Decimal128::from_str(&*f64::to_string(&location.lat)).unwrap(),
                arcade_lng: Decimal128::from_str(&*f64::to_string(&location.lng)).unwrap(),
                arcade_pos: Some(location.to_point()),
                created_at: time,
            };
            arcades.push(arcade);
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    Ok(arcades)
}
pub async fn scrape_arcades() -> Result<(), Box<dyn Error>> {
    info!("开始爬取华立官网机厅");
    let content;
    {
        let browser = Browser::new(LaunchOptions::default_builder().headless(true).build()?)?;
        let tab = browser.new_tab()?;
        tab.navigate_to("http://wc.wahlap.net/maidx/location/index.html")?;
        tab.wait_until_navigated()?;
        wait_for_store_list(&tab)?;
        content = tab.get_content()?;
    }
    match parse_store_list(&content).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
