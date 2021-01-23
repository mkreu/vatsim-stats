use crate::storage::{Flight, FlightStorage};
use chrono::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::RwLock;
use warp::{reply::Json, Filter};

pub async fn run(flight_storage: Arc<RwLock<FlightStorage>>) -> anyhow::Result<()> {
    info!("starting up...");

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
