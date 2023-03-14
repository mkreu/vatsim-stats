use crate::{datafeed, storage::FlightStorage};
use anyhow::Context;
use chrono::prelude::*;
use log::{debug, info};
use std::{path::Path, process::Command};
use walkdir::WalkDir;

const COMPRESSED_FOLDER: &str = "compressed";
const DOWNLOAD_FOLDER: &str = "download";

pub fn import_new(data_path: impl AsRef<Path>) -> anyhow::Result<FlightStorage> {
    let mut flight_storage = FlightStorage::default();
    import_append(&mut flight_storage, data_path)?;
    Ok(flight_storage)
}

pub fn import_append(
    flight_storage: &mut FlightStorage,
    data_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let since = flight_storage.last_ts();
    for compressed in WalkDir::new(data_path.as_ref().join(COMPRESSED_FOLDER))
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .context("compressed dir entry error")?
        .into_iter()
    {
        if compressed.file_type().is_dir() {
            continue;
        }
        if Utc
            .from_utc_datetime(
                &NaiveDate::parse_from_str(
                    compressed
                        .file_name()
                        .to_string_lossy()
                        .trim_end_matches(".tar.xz"),
                    "%Y%m%d",
                )
                .with_context(|| {
                    format!(
                        "failed to parse file name as date: {}",
                        compressed.file_name().to_string_lossy()
                    )
                })?

            .and_hms_opt(0, 0, 0).unwrap(),
            )
            <= since
        {
            continue;
        }

        info!("importing {}", compressed.path().to_string_lossy());
        Command::new("./extract.sh")
            .arg(compressed.path())
            .spawn()?
            .wait()?;

        import_single(flight_storage, "temp", since)?;
    }
    info!("importing from downloaded");
    import_single(
        flight_storage,
        data_path.as_ref().join(DOWNLOAD_FOLDER),
        since,
    )?;

    Ok(())
}

fn import_single(
    flight_storage: &mut FlightStorage,
    from_dir: impl AsRef<Path>,
    since: DateTime<Utc>,
) -> anyhow::Result<()> {
    for entry in WalkDir::new(from_dir)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| entry.file_type().is_file())
    {
        if Utc.from_utc_datetime(
            &NaiveDateTime::parse_from_str(
                entry
                    .file_name()
                    .to_string_lossy()
                    .trim_end_matches(".json"),
                "%Y%m%d%H%M%S",
            )
            .with_context(|| {
                format!(
                    "failed to parse file name as date: {}",
                    entry.file_name().to_string_lossy()
                )
            })?,
        ) <= since
        {
            continue;
        }

        debug!("appending {:?}", entry.path());
        let datafeed = datafeed::read(entry.path())?;
        flight_storage.append(datafeed);
    }
    Ok(())
}
