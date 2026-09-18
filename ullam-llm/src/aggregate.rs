use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlightAggregation {
    pub schema_version: String,
    pub flight: FlightInfo,
    pub phases: Vec<FlightPhase>,
    pub events: Vec<Event>,
    pub observations: Vec<Observation>,
    pub correlations: Vec<Correlation>,
    pub signals: std::collections::HashMap<String, Signal>,
    pub coverage: Coverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlightInfo {
    pub start_time: f64,
    pub duration: f64,
    pub vehicle_type: Option<String>,
    pub firmware: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimeRange {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlightPhase {
    #[serde(rename = "type")]
    pub phase_type: PhaseType,
    pub time_range: TimeRange,
    pub mode: Option<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PhaseType {
    Preflight,
    Takeoff,
    Climb,
    Cruise,
    Maneuver,
    Descent,
    Landing,
    Postflight,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Event {
    pub timestamp: Duration,
    #[serde(rename = "type")]
    pub event_type: String,
    pub description: String,
    pub severity: Severity,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Observation {
    pub time_range: TimeRange,
    pub category: ObservationCategory,
    pub description: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationCategory {
    Attitude,
    Navigation,
    Estimator,
    Gps,
    Imu,
    Vibration,
    Power,
    Rc,
    Motor,
    Control,
    System,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Correlation {
    pub time_range: TimeRange,
    pub input: String,
    pub response: String,
    pub relationship: String,
    pub lag_seconds: Option<f64>,
    /// Correlation coefficient, normally [-1, 1].
    pub strength: Option<f64>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Signal {
    pub message: String,
    pub field: String,
    pub unit: Option<String>,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub count: u64,
    pub first: Option<f64>,
    pub last: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Coverage {
    pub available_messages: Vec<String>,
    pub missing_messages: Vec<String>,
}
