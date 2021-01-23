use std::{sync::Arc, time::Duration};

use anyhow::Context;
use tokio::fs;

use log::{info, warn};
use tokio::sync::RwLock;
use tokio::time;
use vatsim_stats::{importer, server, storage::FlightStorage, vatdl};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let mut flight_storage: FlightStorage = fs::read_to_string("storage/flights.ron")
        .await
        .context("placeholder")
        .and_then(|obj| ron::from_str(&obj).context("placeholder"))
        .unwrap_or_default();

    info!("importing data...");
    let last_ts = flight_storage.last_ts().clone();
    importer::import_since(&mut flight_storage, "data", last_ts).expect("failed to import data");

    let flight_storage = Arc::new(RwLock::new(flight_storage));
    info!("flight data loaded");

    tokio::spawn(downloader(flight_storage.clone()));
    server::run(flight_storage).await.unwrap();
}

async fn downloader(storage: Arc<RwLock<FlightStorage>>) {
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        match vatdl::run_download().await {
            Ok(datafeed) => {
                let mut guard = storage.write().await;
                info!(
                    "downloaded datafeed with time: {}",
                    datafeed.general.update_timestamp
                );
                if datafeed.general.update_timestamp > guard.last_ts() {
                    guard.append(datafeed);
                    if let Err(err) =
                        fs::write("storage/flights.ron", ron::to_string(&*guard).unwrap()).await
                    {
                        warn!("could not save storage: {}", err)
                    } else {
                        info!("appended datafeed and saved storage to file")
                    }
                }
            }
            Err(err) => {
                warn!("error downloading datafeed: {}", err)
            }
        }
    }
}
