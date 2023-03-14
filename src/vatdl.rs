use anyhow::{anyhow, Context};
use chrono::prelude::*;
use log::{info, warn};
use reqwest::Client;
use std::{error::Error, path::Path, time::Duration as StdDuration};
use tokio::{fs, process::Command};

use crate::datafeed::Datafeed;

const DOWNLOAD_DIR: &str = "data/download";
const COMPRESSED_DIR: &str = "data/compressed";

pub async fn run_download() -> anyhow::Result<Datafeed> {
    //Step 1: Download the status index file and extract datafeed urls
    let client = Client::builder()
        .timeout(StdDuration::from_secs(20))
        .build()?;
    let status_file = Path::new("status.txt");
    if !status_file.exists() {
        info!("No status file found. Downloading...");
        download_status(&client).await?;
    }
    let urls = read_status().await?;
    info!("Read urls from status file");

    // There is currently only one url, therefore shuffling is pointless
    // urls.shuffle(&mut thread_rng());
    let dl_results = urls
        .iter()
        .chain(urls.iter())
        .chain(urls.iter())
        .take(3)
        .map(|url| download_datafeed(&client, url));

    let mut datafeed_res: anyhow::Result<Datafeed> = Err(anyhow!("no datafeed url exists"));
    for dl_req in dl_results {
        match dl_req.await {
            Ok(res) => match write_file(&res).await {
                Ok(()) => {
                    datafeed_res = serde_json::from_str(&res)
                        .context("failed to parse downloaded json, saving anyway");
                    break;
                }
                Err(err) => {
                    warn!(
                        "failed to save downloaded file trying another server... : {}",
                        &err
                    );
                    datafeed_res = Err(anyhow!("{}", err));
                }
            },
            Err(err) => {
                warn!("download failed, trying another server... : {}", &err);
                datafeed_res = Err(anyhow!("{}", err));
            }
        }
        info!("Waiting 5 seconds before next download");
        tokio::time::sleep(StdDuration::from_secs(5)).await;
    }
    datafeed_res
}

pub async fn run_compress() -> anyhow::Result<()> {
    //create dir for compressed files if it does not exist
    fs::create_dir_all(COMPRESSED_DIR).await?;
    let today = Utc::now().format("%Y%m%d").to_string();

    let mut dirs = fs::read_dir(DOWNLOAD_DIR).await?;
    while let Some(dir_entry) = dirs.next_entry().await? {
        let path = dir_entry
            .file_name()
            .to_str()
            .context("filename not valid UTF-8")?
            .to_string();
        if path.as_str() < today.as_str() {
            info!("compressing directory '{}'", path);
            compress_day(&path).await?;
            info!("removing directory '{}'", path);
            fs::remove_dir_all(dir_entry.path()).await?;
        }
    }
    Ok(())
}

async fn write_file(datafeed: &str) -> Result<(), Box<dyn Error>> {
    let ts = extract_timestamp(datafeed)?;
    info!("Timestamp: {}", ts);
    let path = timestamp_to_path(&ts);
    fs::create_dir_all(
        Path::new(&path)
            .parent()
            .expect("Path from timestamp was empty"),
    )
    .await?;
    fs::write(&path, datafeed).await?;
    Ok(())
}

async fn read_status() -> anyhow::Result<Vec<String>> {
    let urls = fs::read_to_string("status.txt")
        .await?
        .lines()
        .filter(|line| line.starts_with("json3="))
        .map(|line| String::from(line.split_at("json3=".len()).1.trim()))
        .collect::<Vec<String>>();
    Ok(urls)
}

async fn download_status(client: &Client) -> anyhow::Result<()> {
    let bytes = client
        .get("https://status.vatsim.net/")
        .send()
        .await?
        .bytes()
        .await?;
    fs::write("status.txt", &bytes).await?;
    info!("Downloaded status file");
    Ok(())
}

async fn download_datafeed(client: &Client, url: &str) -> anyhow::Result<String> {
    info!("Downloading vatsim status from: {}", url);
    let datafeed = client.get(url).send().await?.text().await?;
    info!("Sucessfully downloaded datafeed from: {}", url);
    Ok(datafeed)
}

async fn compress_day(date: &str) -> anyhow::Result<()> {
    if Path::new(&format!("compressed/{}.tar.xz", date)).exists() {
        Err(anyhow!(
            "archive file does already exist, doing nothing to avoid data loss"
        ))?;
    }
    let child = Command::new("tar")
        .arg("-caf")
        .arg(format!("{COMPRESSED_DIR}/{date}.tar.xz"))
        .arg("-C")
        .arg(DOWNLOAD_DIR)
        .arg(date)
        .spawn();
    // Await until the future (and the command) completes
    let status = child?.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("failed to execute tar command"))?
    }
}

fn timestamp_to_path(ts: &str) -> String {
    format!(
        "{root}/{year}{month}{day}/{hour}/{ts}.json",
        root = DOWNLOAD_DIR,
        year = &ts[0..4],
        month = &ts[4..6],
        day = &ts[6..8],
        hour = &ts[8..10],
        ts = ts
    )
}

fn extract_timestamp(text: &str) -> Result<String, Box<dyn Error>> {
    let ts = serde_json::from_str::<Datafeed>(text)?.general.update;
    if ts.len() != 14 {
        Err(format!("invalid timestamp: {}", &ts))?
    } else {
        Ok(ts)
    }
}
