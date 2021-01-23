use std::fs;

use vatsim_stats::{importer, storage::FlightStorage};

fn main() {
    pretty_env_logger::init();
    let mut flight_storage = FlightStorage::new();
    importer::import(&mut flight_storage, "data").expect("failed to import");
    fs::write(
        "storage/flights.ron",
        ron::to_string(&flight_storage).unwrap(),
    )
    .unwrap();
}
