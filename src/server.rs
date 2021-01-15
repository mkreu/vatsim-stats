use crate::{
    datafeed::{self, Datafeed},
    storage::FlightStorage,
};
use chrono::prelude::*;
use log::{info, warn};
use serde::Deserialize;
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
    let hello = warp::path::path("flights")
        .and(warp::path::end())
        .and(warp::query())
        .and(with_flights)
        .and_then(
            |query: Query, flights: Arc<RwLock<FlightStorage>>| async move {
                let result: Result<String, Infallible> = Ok(flights
                    .read()
                    .await
                    .flights()
                    .filter(|f| match (&query.arrival_apt, &query.departure_apt) {
                        (Some(ref arr), Some(ref dep)) if arr == dep => {
                            f.arrival == arr || f.departure == dep
                        }
                        (Some(ref arr), Some(ref dep)) => f.arrival == arr && f.departure == dep,
                        (Some(ref arr), None) => f.arrival == arr,
                        (None, Some(ref dep)) => f.departure == dep,
                        (None, None) => false,
                    })
                    .filter(|f| {
                        f.arrival_time
                            .filter(|time| {
                                time >= &query.arrival_time_from.unwrap_or(*time)
                                    && time <= &query.arrival_time_to.unwrap_or(*time)
                            })
                            .is_some()
                    })
                    .filter(|f| {
                        f.departure_time
                            .filter(|time| {
                                time >= &query.departure_time_from.unwrap_or(*time)
                                    && time <= &query.departure_time_to.unwrap_or(*time)
                            })
                            .is_some()
                    })
                    .map(|f| format!("{}", serde_json::to_string(&f).unwrap()))
                    .collect::<Vec<String>>()
                    .join("\n"));
                result
            },
        );

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

#[derive(Deserialize)]
struct Query {
    arrival_apt: Option<String>,
    departure_apt: Option<String>,
    arrival_time_from: Option<DateTime<Utc>>,
    arrival_time_to: Option<DateTime<Utc>>,
    departure_time_from: Option<DateTime<Utc>>,
    departure_time_to: Option<DateTime<Utc>>,
}

pub trait DataProvider {
    fn data_since(&self, time: DateTime<Utc>) -> Box<dyn Iterator<Item = Datafeed>>;
}
