use std::fs;

use vatsim_stats::importer;

fn main() {
    pretty_env_logger::init();
    let flight_storage = importer::import_new("data").expect("failed to import");
    fs::write(
        "storage/flights.ron",
        ron::to_string(&flight_storage).unwrap(),
    )
    .unwrap();
}
