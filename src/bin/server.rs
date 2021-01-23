use std::{fs, sync::Arc, time::Duration};

use log::{info, warn};
use tokio::sync::RwLock;
use tokio::time;
use vatsim_stats::{datafeed, server, storage::FlightStorage};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let flight_storage: FlightStorage =
        ron::from_str(&fs::read_to_string("storage/flights.ron").unwrap()).unwrap();
    let flight_storage = Arc::new(RwLock::new(flight_storage));
    info!("flight data loaded");
    tokio::spawn(downloader(flight_storage.clone()));
    server::run(flight_storage).await.unwrap();
}
async fn downloader(storage: Arc<RwLock<FlightStorage>>) {
    let mut interval = time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        match datafeed::download().await {
            Ok(datafeed) => {
                let mut guard = storage.write().await;
                info!(
                    "downloaded datafeed with time: {}",
                    datafeed.general.update_timestamp
                );
                if datafeed.general.update_timestamp > guard.last_ts() {
                    guard.append(datafeed);
                }
            }
            Err(err) => {
                warn!("error downloading datafeed: {}", err)
            }
        }
    }
}
