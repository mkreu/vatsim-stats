use chrono::prelude::*;
use serde::Deserialize;
use std::{
    fs::{self},
    io,
    path::Path,
};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Datafeed {
    pub general: General,
    pub pilots: Vec<Pilot>,
}

#[derive(Debug, Deserialize)]
pub struct General {
    pub update: String,
    pub update_timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Pilot {
    pub cid: usize,
    pub callsign: String,
    pub flight_plan: Option<Flightplan>,
    pub logon_time: DateTime<Utc>,
    pub groundspeed: isize,
}

#[derive(Debug, Deserialize)]
pub struct Flightplan {
    pub departure: String,
    pub arrival: String,
}

#[derive(Error, Debug)]
pub enum DatafeedReadError {
    #[error("File Error")]
    IOError(#[from] io::Error),
    #[error("Serde Error")]
    DeserializationError(#[from] serde_json::Error),
}

pub fn read(path: impl AsRef<Path>) -> Result<Datafeed, DatafeedReadError> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}
