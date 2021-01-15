use std::collections::HashMap;

use crate::datafeed::{Datafeed, Pilot};
use chrono::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FlightStorage {
    last_ts: DateTime<Utc>,
    flights: HashMap<FlightKey, FlightData>,
    prev_gs: HashMap<FlightKey, usize>,
}

impl FlightStorage {
    pub fn new() -> Self {
        Self {
            last_ts: Utc.timestamp(0, 0),
            flights: HashMap::new(),
            prev_gs: HashMap::new(),
        }
    }
    pub fn append(&mut self, datafeed: Datafeed) {
        if datafeed.general.update_timestamp <= self.last_ts {
            panic!("you can only append timestamps in monotone increasing order")
        }
        let mut cur_gs = HashMap::new();
        for (cs, fp, gs) in datafeed.pilots.into_iter().filter_map(|p| {
            let Pilot {
                flight_plan,
                callsign,
                groundspeed,
                logon_time,
                ..
            } = p;
            flight_plan.map(|fp| (FlightKey(callsign, logon_time), fp, groundspeed))
        }) {
            let entry = self
                .flights
                .entry(cs.clone())
                .or_insert_with(|| FlightData {
                    departure: fp.departure,
                    arrival: fp.arrival,
                    departure_time: None,
                    arrival_time: None,
                });

            match self.prev_gs.get(&cs) {
                Some(ref gs_old) if **gs_old < 60 => {
                    if gs >= 60 {
                        entry.departure_time = Some(datafeed.general.update_timestamp);
                    }
                }
                Some(ref gs_old) if **gs_old >= 60 => {
                    if gs < 60 {
                        entry.arrival_time = Some(datafeed.general.update_timestamp);
                    }
                }
                _ => {}
            }
            cur_gs.insert(cs, gs);
        }
        self.prev_gs = cur_gs;
    }
    pub fn flights(&self) -> impl Iterator<Item = Flight<'_>> {
        self.flights.iter().map(|(key, data)| Flight {
            callsign: &key.0,
            departure: &data.departure,
            arrival: &data.arrival,
            departure_time: &data.departure_time,
            arrival_time: &data.arrival_time,
        })
    }
    pub fn last_ts(&self) -> DateTime<Utc> {
        self.last_ts
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct FlightKey(String, DateTime<Utc>);
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlightData {
    departure: String,
    arrival: String,
    departure_time: Option<DateTime<Utc>>,
    arrival_time: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Flight<'a> {
    pub callsign: &'a str,
    pub departure: &'a str,
    pub arrival: &'a str,
    pub departure_time: &'a Option<DateTime<Utc>>,
    pub arrival_time: &'a Option<DateTime<Utc>>,
}
