use crate::{DB_NAME, get_mongodb_client};
use headless_chrome::{Browser, LaunchOptions, Tab};

use crate::types::Arcade;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

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
    let geocoder_response: GeocoderResponse = response.json().await.unwrap();
    if geocoder_response.status == 0 {
        if let Some(result) = geocoder_response.result {
            return Ok(Some(result.location));
        }
    }
    Ok(None)
}

async fn parse_store_list(html: &str) -> Result<Vec<Arcade>, Box<dyn Error>> {
    let document = Html::parse_document(html);
    let mut arcades = Vec::new();
    let ul_selector = Selector::parse(".store_list").unwrap();
    let li_selector = Selector::parse("li").unwrap();

    let store_name_selector = Selector::parse("span.store_name")?;
    let store_address_selector = Selector::parse("span.store_address")?;

    let mut id_counter = 0;
    for (_, store_list) in document.select(&ul_selector).enumerate() {
        for li in store_list.select(&li_selector) {
            id_counter += 1;

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

            let location = get_geo_location(&store_address).await.unwrap_or_else(|e| {
                eprintln!("获取地理位置信息失败: {}", e);
                None // 出错时返回None
            });
            println!(
                "{}\n{}\n{}\n{}\n\n",
                id_counter,
                store_name,
                store_address,
                match &location {
                    Some(loc) => format!("{}, {}", loc.lat, loc.lng),
                    None => "无地理位置信息".to_string(),
                }
            );
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    Ok(arcades)
}
pub async fn scrape_arcades() -> Result<(), Box<dyn Error>> {
    println!("开始爬取华立官网机厅");
    let content;
    {
        let browser = Browser::new(LaunchOptions::default_builder().headless(true).build()?)?;
        let tab = browser.new_tab()?;
        tab.navigate_to("http://wc.wahlap.net/maidx/location/index.html")?;
        tab.wait_until_navigated()?;
        wait_for_store_list(&tab)?;
        content = tab.get_content()?;
    }
    let _ = parse_store_list(&content).await?;
    Ok(())
}
