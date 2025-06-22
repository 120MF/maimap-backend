use maimap_utils::errors::{Context, Result};
use maimap_utils::types::Arcade;
use std::collections::{HashMap, HashSet};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;

/// 将数据库和网站的机厅名称分别导出到文件，用于调试比对。
pub(crate) async fn export_arcade_names_to_files(
    existing_arcades: &HashMap<String, Arcade>,
    web_arcades: &[(String, String)],
) -> Result<()> {
    info!("正在导出机厅名称用于比对...");

    // 提取数据库中的机厅名称并写入文件
    let existing_names: HashSet<_> = existing_arcades.keys().cloned().collect();
    let mut file_existing = File::create("existing_arcades.txt")
        .await
        .context("创建 existing_arcades.txt 失败")?;
    for name in &existing_names {
        file_existing
            .write_all(format!("{}\n", name).as_bytes())
            .await?;
    }
    info!(
        "已将 {} 个数据库机厅名称导出到 'existing_arcades.txt'",
        existing_names.len()
    );

    // 提取网站上的机厅名称并写入文件
    let web_names: HashSet<_> = web_arcades.iter().map(|(name, _)| name.clone()).collect();
    let mut file_web = File::create("web_arcades.txt")
        .await
        .context("创建 web_arcades.txt 失败")?;
    for name in &web_names {
        file_web.write_all(format!("{}\n", name).as_bytes()).await?;
    }
    info!(
        "已将 {} 个网站机厅名称导出到 'web_arcades.txt'",
        web_names.len()
    );

    Ok(())
}
