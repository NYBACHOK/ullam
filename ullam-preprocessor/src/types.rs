use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlightPhase {
    #[serde(rename = "STABILIZE")]
    Stabilize,
    #[serde(rename = "LOITER")]
    Loiter,
    #[serde(rename = "RTL")]
    Rtl,
    #[serde(rename = "FLIP")]
    Flip,
    #[serde(rename = "CRASHING")]
    Crashing,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub count: usize,
    pub oscillation_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowIrRecord {
    pub msg: String,
    pub stats: HashMap<String, NumericStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotIrRecord {
    pub msg: String,
    pub fields: Vec<(String, ullam_parser::LogValue)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PreprocessedLogItem {
    pub timestamp: Duration,
    pub duration: Duration,
    pub windows: Vec<WindowIrRecord>,
    pub snapshots: Vec<SnapshotIrRecord>,
}
