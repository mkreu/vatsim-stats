use std::fs;

use chrono::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use storage::FlightStorage;

mod datafeed;
mod storage;

fn main() {
    let datafeed = datafeed::read("data/20201215/15/20201215150154.json").unwrap();
    println!("{} pilots", datafeed.pilots.len());
    println!("{:?}", datafeed.pilots[350]);

    let mut flight_storage = FlightStorage::new();

    for entry in WalkDir::new("data/20201215")
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .filter(|entry| entry.file_type().is_file())
    {
        println!("{:?}", entry.path());
        let datafeed = datafeed::read(entry.path()).unwrap();
        flight_storage.append(datafeed);
    }

    /*
    let mut departures: Vec<_> = flights
        .iter()
        .filter(|(_cs, f)| (f.departure == "EDDM" && f.departure_time.is_some()))
        .map(|(cs, f)| (cs, f.departure_time.unwrap()))
        .collect();
    departures.sort_by_key(|(_cs, time)| *time);
    for line in &departures {
        println! {"{:?}", line}
    }
    dbg!(departures.len());
    let mut departures: Vec<_> = flights
        .iter()
        .filter(|(_cs, f)| (f.arrival == "EDDM" && f.arrival_time.is_some()))
        .map(|(cs, f)| (cs, f.arrival_time.unwrap()))
        .collect();
    departures.sort_by_key(|(_cs, time)| *time);
    for line in &departures {
        println! {"{:?}", line}
    }
    dbg!(departures.len());*/

    fs::write(
        "storage/flights.ron",
        ron::ser::to_string(&flight_storage).unwrap(),
    )
    .unwrap();
}

#[derive(Debug, Serialize)]
struct Flight {
    //callsign: String,
    departure: String,
    arrival: String,
    departure_time: Option<DateTime<Utc>>,
    arrival_time: Option<DateTime<Utc>>,
}
