use std::{fs, process::Command};

use chrono::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use storage::FlightStorage;

mod datafeed;
mod storage;

fn main() {
    let mut flight_storage = FlightStorage::new();

    for compressed in WalkDir::new("data").sort_by(|a, b| a.file_name().cmp(b.file_name())) {
        let compressed = compressed.unwrap();
        if compressed.file_type().is_dir() {
            continue;
        }
        Command::new("./extract.sh")
            .arg(dbg!(&compressed.path()))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        for entry in WalkDir::new("temp")
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
