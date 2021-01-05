use std::{fs, process::Command};

use walkdir::WalkDir;

use vatsim_stats::{datafeed, storage::FlightStorage};

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

    fs::write(
        "storage/flights.ron",
        ron::ser::to_string(&flight_storage).unwrap(),
    )
    .unwrap();
}
