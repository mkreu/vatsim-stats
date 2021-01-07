use crate::{
    datafeed::{self, Datafeed},
    storage::FlightStorage,
};
use chrono::prelude::*;
use log::{info, warn};
use std::time::Duration;
use std::{convert::Infallible, fs, sync::Arc};
use tokio::sync::RwLock;
use warp::Filter;

async fn downloader(storage: Arc<RwLock<FlightStorage>>) {
    loop {
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
        tokio::time::delay_for(Duration::from_secs(30)).await;
    }
}

pub async fn run(data_provider: impl DataProvider) -> anyhow::Result<()> {
    info!("starting up...");
    let mut flight_storage: FlightStorage =
        ron::from_str(&fs::read_to_string("storage/flights.ron")?)?;
    for datafeed in data_provider.data_since(flight_storage.last_ts()) {
        flight_storage.append(datafeed);
    }
    let flight_storage = Arc::new(RwLock::new(flight_storage));
    info!("flight data loaded");
    tokio::spawn(downloader(flight_storage.clone()));

    let with_flights = warp::any().map(move || flight_storage.clone());
    let hello = warp::path!("flights" / String / String)
        .and(with_flights)
        .and_then(|dep, arr, flights: Arc<RwLock<FlightStorage>>| async move {
            let result: Result<String, Infallible> = Ok(flights
                .read()
                .await
                .flights()
                .filter(|f| f.departure == dep && f.arrival == arr)
                .map(|f| format!("{}", f.callsign))
                .collect::<Vec<String>>()
                .join("\n"));
            result
        });

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

pub trait DataProvider {
    fn data_since(&self, time: DateTime<Utc>) -> Box<dyn Iterator<Item = Datafeed>>;
}
