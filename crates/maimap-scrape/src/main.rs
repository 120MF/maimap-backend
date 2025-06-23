use std::str::FromStr;
mod cleanup;
mod export_hashmap;
mod geo_location;

use crate::geo_location::get_geo_location;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::collections::{HashMap, HashSet};

use maimap_utils::backup::backup_database;
use maimap_utils::db::{
    DateTime, Decimal128, ensure_mongodb_connected, get_max_arcade_id, insert_many_arcades,
};
use maimap_utils::env::check_required_env_vars;
use maimap_utils::errors::{AppError, Context, Result};
use maimap_utils::types::Arcade;

use crate::cleanup::{
    convert_lat_lng_to_decimal128, convert_null_dead_to_bool, normalize_name,
    remove_duplicate_arcades,
};
use crate::export_hashmap::export_arcade_names_to_files;
use scraper::{Html, Selector};
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    info!("执行定时爬取华立机厅任务");
    check_required_env_vars();
    ensure_mongodb_connected().await;
    match remove_duplicate_arcades().await {
        Ok(_) => {
            info!("清理数据库完成！");
        }
        Err(e) => {
            error!("清理数据库失败！{}", e);
            return;
        }
    }
    match convert_lat_lng_to_decimal128().await {
        Ok(_) => {
            info!("转换经纬度数据成功！");
        }
        Err(e) => {
            error!("转换经纬度数据失败！{}", e);
            return;
        }
    }
    match convert_null_dead_to_bool().await {
        Ok(_) => {
            info!("转换 arcade_dead 数据成功！");
        }
        Err(e) => {
            error!("转换 arcade_dead 数据失败！{}", e);
            return;
        }
    }
    match scrape_arcades().await {
        Ok(_) => {
            info!("爬取任务成功！");
        }
        Err(e) => {
            error!("爬取任务失败！{}", e);
            return;
        }
    }

    match backup_database().await {
        Ok(_) => info!("备份数据库成功！"),
        Err(e) => error!("备份数据库失败！{}", e),
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

    let existing_arcades = get_existing_arcades().await?;
    info!("从数据库获取到 {} 个已存在机厅", existing_arcades.len());

    let web_arcades = parse_all_store_list(&content).await?;
    info!("从网站解析到 {} 个机厅", web_arcades.len());

    export_arcade_names_to_files(&existing_arcades, &web_arcades).await?;

    process_arcade_data(existing_arcades, web_arcades).await
}

async fn get_existing_arcades() -> Result<HashMap<String, Arcade>> {
    use maimap_utils::db::get_all_arcades;

    let arcades = get_all_arcades().await?;
    let mut arcade_map = HashMap::new();

    for mut arcade in arcades {
        let normalized_name = normalize_name(&arcade.arcade_name);
        if arcade_map.contains_key(&normalized_name) {
            warn!(
                "发现规范化后重复的机厅名称: '{}' (原名: '{}'), 将跳过此条目。",
                normalized_name, arcade.arcade_name
            );
            continue;
        }
        arcade.arcade_name = normalized_name.clone();
        arcade_map.insert(normalized_name, arcade);
    }

    Ok(arcade_map)
}

fn wait_for_store_list(tab: &Tab) -> Result<()> {
    const MIN_ARCADES: usize = 2000;
    const MAX_RETRIES: u32 = 5;
    const POLLING_TIMEOUT: Duration = Duration::from_secs(90);
    const POLLING_INTERVAL: Duration = Duration::from_secs(2);

    for attempt in 1..=MAX_RETRIES {
        info!("开始加载机厅列表 (尝试 {}/{})", attempt, MAX_RETRIES);

        if attempt > 1 {
            info!("机厅数量不足，重新加载页面...");
            tab.reload(false, None).context("页面重新加载失败")?;
            tab.wait_until_navigated()
                .context("等待页面重新加载后导航失败")?;
        }

        tab.wait_for_element_with_custom_timeout(".store_list", Duration::from_secs(60))
            .context("等待 .store_list 容器元素超时")?;
        info!("列表容器已出现，开始轮询检查机厅数量...");

        let start_time = std::time::Instant::now();
        loop {
            let js_script = "document.querySelectorAll('.store_list li').length";
            let result = tab
                .evaluate(js_script, false)
                .context("执行 JavaScript 数量检查失败")?;
            let count = match result.value {
                Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
                _ => 0,
            };
            if count >= MIN_ARCADES {
                info!("成功加载 {} 个机厅，数量符合预期。", count);
                return Ok(());
            }
            if start_time.elapsed() > POLLING_TIMEOUT {
                warn!(
                    "轮询超时 ({}s)，当前加载了 {} 个机厅，未达到 {} 个。准备重试...",
                    POLLING_TIMEOUT.as_secs(),
                    count,
                    MIN_ARCADES
                );
                break;
            }

            info!("当前已加载 {}/{} 个机厅，继续等待...", count, MIN_ARCADES);
            std::thread::sleep(POLLING_INTERVAL);
        }
    }

    Err(AppError::Scrape(format!(
        "经过 {} 次尝试后，仍未能加载到至少 {} 个机厅。",
        MAX_RETRIES, MIN_ARCADES
    ))
    .into())
}

async fn parse_all_store_list(html: &str) -> Result<Vec<(String, String)>> {
    let document = Html::parse_document(html);
    let mut arcade_info = Vec::new();

    let ul_selector = Selector::parse(".store_list").map_err(|e| AppError::Parse(e.to_string()))?;
    let li_selector = Selector::parse("li").map_err(|e| AppError::Parse(e.to_string()))?;
    let store_name_selector =
        Selector::parse("span.store_name").map_err(|e| AppError::Parse(e.to_string()))?;
    let store_address_selector =
        Selector::parse("span.store_address").map_err(|e| AppError::Parse(e.to_string()))?;

    for store_list in document.select(&ul_selector) {
        for li in store_list.select(&li_selector) {
            let store_name_raw = li
                .select(&store_name_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .ok_or_else(|| AppError::Parse("store_name".to_string()))?;

            let store_name = normalize_name(&store_name_raw);

            let store_address = li
                .select(&store_address_selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .ok_or_else(|| AppError::Parse("store_address".to_string()))?;

            arcade_info.push((store_name, store_address));
        }
    }

    Ok(arcade_info)
}

async fn process_arcade_data(
    existing_arcades: HashMap<String, Arcade>,
    web_arcades: Vec<(String, String)>,
) -> Result<()> {
    let time = DateTime::now();
    let max_id = get_max_arcade_id().await?;

    // 用于标记数据库中已处理的机厅
    let mut processed_arcade_names = HashSet::new();
    let mut arcades_to_update = Vec::new();
    let mut new_arcades = Vec::new();
    let mut id_counter = max_id;

    // 处理网站上的机厅
    for (name, address) in web_arcades {
        if let Some(existing) = existing_arcades.get(&name) {
            // 机厅已存在，检查地址是否变化或机厅是否已关闭
            processed_arcade_names.insert(name.clone());

            if existing.arcade_address != address {
                // 地址有变动，需要获取新的地理位置
                info!(
                    "机厅地址或状态有变，准备更新: {}，旧地址：{}，新地址：{}",
                    name, existing.arcade_address, address
                );
                let location = get_geo_location(&address).await?;
                tokio::time::sleep(Duration::from_millis(1000)).await;

                let updated = Arcade {
                    arcade_id: existing.arcade_id,
                    arcade_name: existing.arcade_name.clone(),
                    arcade_address: address,
                    arcade_dead: existing.arcade_dead,
                    arcade_cost: existing.arcade_cost,
                    arcade_count: existing.arcade_count,
                    arcade_lat: Decimal128::from_str(&location.lat.to_string())?,
                    arcade_lng: Decimal128::from_str(&location.lng.to_string())?,
                    arcade_pos: Some(location.to_point()),
                    created_at: existing.created_at,
                };

                arcades_to_update.push(updated);
                info!("更新完成");
            }
        } else {
            // 新机厅，需要获取地理位置
            info!("发现新机厅，准备获取地理位置: {}", name);
            let location = get_geo_location(&address).await?;
            tokio::time::sleep(Duration::from_millis(1000)).await;

            id_counter += 1;

            let arcade = Arcade {
                arcade_id: id_counter,
                arcade_name: name.clone(),
                arcade_address: address,
                arcade_dead: false,
                arcade_cost: None,
                arcade_count: None,
                arcade_lat: Decimal128::from_str(&location.lat.to_string())?,
                arcade_lng: Decimal128::from_str(&location.lng.to_string())?,
                arcade_pos: Some(location.to_point()),
                created_at: time,
            };

            new_arcades.push(arcade);
            info!("新增机厅：ID {}，名称 {}", id_counter, name);
        }
    }

    // 标记已关闭的机厅
    let mut closed_arcades = Vec::new();
    for (name, arcade) in &existing_arcades {
        if !processed_arcade_names.contains(name) && !arcade.arcade_dead {
            let closed = Arcade {
                arcade_id: arcade.arcade_id,
                arcade_name: arcade.arcade_name.clone(),
                arcade_address: arcade.arcade_address.clone(),
                arcade_dead: true,
                arcade_cost: arcade.arcade_cost,
                arcade_count: arcade.arcade_count,
                arcade_lat: arcade.arcade_lat,
                arcade_lng: arcade.arcade_lng,
                arcade_pos: arcade.arcade_pos.clone(),
                created_at: arcade.created_at,
            };
            closed_arcades.push(closed);
            info!("标记已关闭机厅：ID {}，名称 {}", arcade.arcade_id, name);
        }
    }

    // 执行数据库操作
    let arcades_to_update_len = arcades_to_update.len();
    if !arcades_to_update.is_empty() {
        update_arcades(&arcades_to_update).await?;
    }

    let new_arcades_len = new_arcades.len();
    if !new_arcades.is_empty() {
        insert_many_arcades(new_arcades).await?;
    }

    let closed_arcades_len = closed_arcades.len();
    if !closed_arcades.is_empty() {
        update_arcades(&closed_arcades).await?;
    }

    info!(
        "处理完成：更新 {} 个机厅，新增 {} 个机厅，标记关闭 {} 个机厅",
        arcades_to_update_len, new_arcades_len, closed_arcades_len
    );

    Ok(())
}

async fn update_arcades(arcades: &[Arcade]) -> Result<()> {
    use maimap_utils::db::update_arcade;

    for arcade in arcades {
        update_arcade(arcade).await?;
    }

    Ok(())
}
