use std::fmt::{self, Display, Formatter};
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

impl Display for FlightAggregation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "FLIGHT")?;
        writeln!(f, "  schema={}", self.schema_version)?;
        writeln!(f, "  start={:.3}s", self.flight.start_time)?;
        writeln!(f, "  duration={:.3}s", self.flight.duration)?;

        if let Some(vehicle_type) = &self.flight.vehicle_type {
            writeln!(f, "  vehicle={vehicle_type}")?;
        }

        if let Some(firmware) = &self.flight.firmware {
            writeln!(f, "  firmware={firmware}")?;
        }

        if !self.phases.is_empty() {
            writeln!(f)?;
            writeln!(f, "PHASES")?;

            for phase in &self.phases {
                writeln!(f, "{phase}")?;
            }
        }

        if !self.events.is_empty() {
            writeln!(f)?;
            writeln!(f, "EVENTS")?;

            for event in &self.events {
                writeln!(f, "{event}")?;
            }
        }

        if !self.observations.is_empty() {
            writeln!(f)?;
            writeln!(f, "OBSERVATIONS")?;

            for observation in &self.observations {
                writeln!(f, "{observation}")?;
            }
        }

        if !self.correlations.is_empty() {
            writeln!(f)?;
            writeln!(f, "CORRELATIONS")?;

            for correlation in &self.correlations {
                writeln!(f, "{correlation}")?;
            }
        }

        if !self.signals.is_empty() {
            writeln!(f)?;
            writeln!(f, "SIGNALS")?;

            let mut signals: Vec<_> = self.signals.values().collect();
            signals.sort_unstable_by(|a, b| {
                a.message
                    .cmp(&b.message)
                    .then_with(|| a.field.cmp(&b.field))
            });

            for signal in signals {
                writeln!(f, "{signal}")?;
            }
        }

        writeln!(f)?;
        writeln!(f, "COVERAGE")?;
        writeln!(
            f,
            "  available={}",
            self.coverage.available_messages.join(",")
        )?;

        if !self.coverage.missing_messages.is_empty() {
            writeln!(f, "  missing={}", self.coverage.missing_messages.join(","))?;
        }

        Ok(())
    }
}

impl Display for FlightInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "  start={:.3}s", self.start_time)?;
        writeln!(f, "  duration={:.3}s", self.duration)?;

        if let Some(vehicle_type) = &self.vehicle_type {
            writeln!(f, "  vehicle={vehicle_type}")?;
        }

        if let Some(firmware) = &self.firmware {
            writeln!(f, "  firmware={firmware}")?;
        }

        Ok(())
    }
}

impl Display for FlightPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PHASE {} {:.3}..{:.3}s",
            self.phase_type, self.time_range.start, self.time_range.end,
        )?;

        if let Some(mode) = &self.mode {
            write!(f, " mode={mode}")?;
        }

        if !self.evidence.is_empty() {
            write!(f, " evidence={}", self.evidence.join("; "))?;
        }

        writeln!(f)
    }
}

impl Display for PhaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Preflight => "preflight",
            Self::Takeoff => "takeoff",
            Self::Climb => "climb",
            Self::Cruise => "cruise",
            Self::Maneuver => "maneuver",
            Self::Descent => "descent",
            Self::Landing => "landing",
            Self::Postflight => "postflight",
            Self::Unknown => "unknown",
        };

        f.write_str(value)
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EVENT {:.3}s {} severity={}",
            self.timestamp.as_secs_f64(),
            self.event_type,
            self.severity,
        )?;

        write!(f, " {}", self.description)?;

        if !self.evidence.is_empty() {
            write!(f, " evidence={}", self.evidence.join("; "))?;
        }

        writeln!(f)
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        };

        f.write_str(value)
    }
}

impl Display for Observation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OBS {:.3}..{:.3}s category={}",
            self.time_range.start, self.time_range.end, self.category,
        )?;

        write!(f, " {}", self.description)?;

        if !self.evidence.is_empty() {
            write!(f, " evidence={}", self.evidence.join("; "))?;
        }

        writeln!(f)
    }
}

impl Display for ObservationCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Attitude => "attitude",
            Self::Navigation => "navigation",
            Self::Estimator => "estimator",
            Self::Gps => "gps",
            Self::Imu => "imu",
            Self::Vibration => "vibration",
            Self::Power => "power",
            Self::Rc => "rc",
            Self::Motor => "motor",
            Self::Control => "control",
            Self::System => "system",
            Self::Other => "other",
        };

        f.write_str(value)
    }
}

impl Display for Correlation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CORR {:.3}..{:.3}s {} -> {} relationship={}",
            self.time_range.start,
            self.time_range.end,
            self.input,
            self.response,
            self.relationship,
        )?;

        if let Some(lag) = self.lag_seconds {
            write!(f, " lag={lag:.3}s")?;
        }

        if let Some(strength) = self.strength {
            write!(f, " strength={strength:.3}")?;
        }

        if !self.evidence.is_empty() {
            write!(f, " evidence={}", self.evidence.join("; "))?;
        }

        writeln!(f)
    }
}

impl Display for Signal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SIGNAL {}.{}", self.message, self.field)?;

        if let Some(unit) = &self.unit {
            write!(f, " unit={unit}")?;
        }

        write!(
            f,
            " {:.6}..{:.6} μ={:.6} o={:.6} n={}",
            self.min, self.max, self.mean, self.stddev, self.count,
        )?;

        if let Some(first) = self.first {
            write!(f, " first={first:.6}")?;
        }

        if let Some(last) = self.last {
            write!(f, " last={last:.6}")?;
        }

        writeln!(f)
    }
}

impl Display for Coverage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "  available={}", self.available_messages.join(","))?;

        if !self.missing_messages.is_empty() {
            writeln!(f, "  missing={}", self.missing_messages.join(","))?;
        }

        Ok(())
    }
}
