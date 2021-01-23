use std::{sync::Arc, time::Duration};

use anyhow::Context;
use log::{error, info};
use tokio::{fs, sync::RwLock, time};

use vatsim_stats::{importer, server, storage::FlightStorage};

const IMPORT_INTERVAL: u64 = 10 * 60;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let mut flight_storage: FlightStorage = fs::read_to_string("storage/flights.ron")
        .await
        .context("placeholder")
        .and_then(|obj| ron::from_str(&obj).context("placeholder"))
        .unwrap_or_default();

    info!("importing data...");
    importer::import_append(&mut flight_storage, "data").expect("failed to import data");
    if let Err(err) = fs::write(
        "storage/flights.ron",
        ron::to_string(&flight_storage).unwrap(),
    )
    .await
    {
        error!("could not save storage: {}", err)
    } else {
        info!("saved storage");
    }

    let flight_storage = Arc::new(RwLock::new(flight_storage));
    info!("flight data loaded");

    tokio::spawn(import_task(flight_storage.clone()));
    server::run(flight_storage).await.unwrap();
}

async fn import_task(storage: Arc<RwLock<FlightStorage>>) {
    let mut interval = time::interval(Duration::from_secs(IMPORT_INTERVAL));
    loop {
        interval.tick().await;
        let mut guard = storage.write().await;
        if let Err(err) = importer::import_append(&mut *guard, "data") {
            error!("could not append to storage: {}", err);
            continue;
        }
        if let Err(err) = fs::write("storage/flights.ron", ron::to_string(&*guard).unwrap()).await {
            error!("could not save storage: {}", err)
        } else {
            info!("appended and saved storage");
        }
    }
}
