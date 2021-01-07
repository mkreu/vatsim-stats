use std::{fs, sync::Arc};
use vatsim_stats::storage::FlightStorage;
use warp::Filter;
use log::info;

#[tokio::main]
async fn main() {
    run().await.unwrap();
}

async fn run() -> anyhow::Result<()> {
    pretty_env_logger::init();
    info!("starting up...");
    let flight_storage: FlightStorage = ron::from_str(&fs::read_to_string("storage/flights.ron")?)?;
    let flight_storage = Arc::new(flight_storage);
    info!("flight data loaded");
    let with_flights = warp::any().map(move || flight_storage.clone());
    let hello = warp::path!("flights" / String / String)
        .and(with_flights)
        .map(|dep, arr, flights: Arc<FlightStorage>| {
            flights
                .flights()
                .filter(|f| f.departure == dep && f.arrival == arr)
                .map(|f| format!("{}", f.callsign))
                .collect::<Vec<String>>()
                .join("\n")
        });

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
