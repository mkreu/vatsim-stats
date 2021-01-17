use crate::{
    datafeed::{self, Datafeed},
    storage::{Flight, FlightStorage},
};
use chrono::prelude::*;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{convert::Infallible, fs, sync::Arc};
use tokio::sync::RwLock;
use warp::{reply::Json, Filter};

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
    let flights_airport = warp::path!("flights" / "airport")
        .and(warp::path::end())
        .and(warp::query())
        .and(with_flights)
        .and_then(|query: AirportQuery, flights: Arc<RwLock<FlightStorage>>| {
            flights_airport(query, flights)
        });

    warp::serve(flights_airport)
        .run(([127, 0, 0, 1], 3030))
        .await;
    Ok(())
}

#[derive(Deserialize)]
struct AirportQuery {
    icao: String,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
}

pub trait DataProvider {
    fn data_since(&self, time: DateTime<Utc>) -> Box<dyn Iterator<Item = Datafeed>>;
}

async fn flights_airport(
    query: AirportQuery,
    flights: Arc<RwLock<FlightStorage>>,
) -> Result<Json, Infallible> {
    let flights = flights.read().await;
    let departures = flights
        .flights()
        .filter(|flight| flight.departure == query.icao)
        .filter(|flight| {
            flight
                .departure_time
                .filter(|time| time >= &query.since)
                .is_some()
        })
        .filter(|flight| {
            flight
                .departure_time
                .filter(|time| time <= &query.until)
                .is_some()
        })
        .collect();
    let arrivals = flights
        .flights()
        .filter(|flight| flight.arrival == query.icao)
        .filter(|flight| {
            flight
                .arrival_time
                .filter(|time| time >= &query.since)
                .is_some()
        })
        .filter(|flight| {
            flight
                .arrival_time
                .filter(|time| time <= &query.until)
                .is_some()
        })
        .collect();
    Ok(warp::reply::json(&AirportFlights {
        departures,
        arrivals,
    }))
}

#[derive(Serialize)]
struct AirportFlights<'a> {
    departures: Vec<Flight<'a>>,
    arrivals: Vec<Flight<'a>>,
}
