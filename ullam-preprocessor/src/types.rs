use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

fn skip_serializing_if_f64(val: &f64) -> bool {
    !val.is_normal() || *val == 0.0
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MsgName(pub String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldName(pub String);

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

impl FlightPhase {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stabilize => "STABILIZE",
            Self::Loiter => "LOITER",
            Self::Rtl => "RTL",
            Self::Flip => "FLIP",
            Self::Crashing => "CRASHING",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for FlightPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Display for MsgName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Default, schemars::JsonSchema,
)]
pub enum VehicleType {
    #[default]
    Plane,
}

impl std::fmt::Display for VehicleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VehicleType::Plane => write!(f, "plane"),
        }
    }
}

impl std::str::FromStr for VehicleType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s.to_lowercase().as_str() {
            "plane" => Self::Plane,
            _ => return Err("invalid type of venicle"),
        };

        Ok(kind)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericStats {
    #[serde(skip_serializing_if = "skip_serializing_if_f64")]
    pub min: f64,
    #[serde(skip_serializing_if = "skip_serializing_if_f64")]
    pub max: f64,
    #[serde(skip_serializing_if = "skip_serializing_if_f64")]
    pub mean: f64,
    #[serde(skip_serializing_if = "skip_serializing_if_f64")]
    pub stddev: f64,
}

impl NumericStats {
    fn has_displayable_values(&self) -> bool {
        !skip_serializing_if_f64(&self.min)
            || !skip_serializing_if_f64(&self.max)
            || !skip_serializing_if_f64(&self.mean)
            || !skip_serializing_if_f64(&self.stddev)
    }
}

impl std::fmt::Display for NumericStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut separator = "";
        for (name, value) in [
            ("min", self.min),
            ("max", self.max),
            ("mean", self.mean),
            ("stddev", self.stddev),
        ] {
            if skip_serializing_if_f64(&value) {
                continue;
            }

            let formatted = format!("{value:.3}");
            let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
            write!(f, "{separator}{name}={formatted}")?;
            separator = " ";
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowIrRecord {
    pub msg: MsgName,
    pub stats: HashMap<FieldName, NumericStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotedField {
    pub timestamp: Duration,
    pub value: ullam_parser::LogValue,
}

impl std::fmt::Display for SnapshotedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = serde_json::to_string(&self.value).map_err(|_| std::fmt::Error)?;
        write!(f, "{:.3} = {value}", self.timestamp.as_secs_f64())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PreprocessedLog {
    /// Type of venicle that performed flight
    pub vehicle_type: VehicleType,
    /// Processed windows of log with stats and timeline
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<PreprocessedLogItem>,
    /// Plain count of messages
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub msgs_count: HashMap<MsgName, u64>,
    /// Msg name -> field name -> history of field snapshoted values
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub snapshots: HashMap<MsgName, HashMap<FieldName, Vec<SnapshotedField>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PreprocessedLogItem {
    pub timestamp: Duration,
    pub duration: Duration,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub windows: Vec<WindowIrRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<String>,
}

impl std::fmt::Display for WindowIrRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fields: Vec<_> = self
            .stats
            .iter()
            .filter(|(_, stats)| stats.has_displayable_values())
            .collect();
        fields.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));

        if fields.is_empty() {
            return Ok(());
        }

        writeln!(f, "W {}", self.msg)?;

        for (field, stats) in fields {
            writeln!(f, "  {field} {stats}")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for PreprocessedLogItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut windows: Vec<_> = self
            .windows
            .iter()
            .filter(|window| {
                window
                    .stats
                    .values()
                    .any(NumericStats::has_displayable_values)
            })
            .collect();
        windows.sort_unstable_by(|left, right| left.msg.cmp(&right.msg));

        if windows.is_empty() && self.messages.is_empty() {
            return Ok(());
        }

        let start = self.timestamp.as_secs_f64();
        let end = (self.timestamp + self.duration).as_secs_f64();

        writeln!(f, "CHUNK {start:.3}..{end:.3}")?;

        for window in windows {
            write!(f, "{window}")?;
        }

        for message in &self.messages {
            writeln!(f, "E {message}")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for PreprocessedLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "VEHICLE {}", self.vehicle_type)?;

        if !self.msgs_count.is_empty() {
            let mut counts: Vec<_> = self.msgs_count.iter().collect();
            counts.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));

            writeln!(f, "COUNTS")?;
            for (message, count) in counts {
                writeln!(f, "  {message} {count}")?;
            }
        }

        let mut snapshots: Vec<_> = self.snapshots.iter().collect();
        snapshots.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        let snapshots: Vec<_> = snapshots
            .into_iter()
            .filter_map(|(message, fields)| {
                let mut fields: Vec<_> = fields
                    .iter()
                    .filter(|(_, values)| !values.is_empty())
                    .collect();
                fields.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
                (!fields.is_empty()).then_some((message, fields))
            })
            .collect();

        if !snapshots.is_empty() {
            writeln!(f, "SNAPSHOTS")?;
            for (message, fields) in snapshots {
                writeln!(f, "S {message}")?;
                for (field, values) in fields {
                    for value in values {
                        writeln!(f, "  {field} {value}")?;
                    }
                }
            }
        }

        for item in &self.items {
            write!(f, "{item}")?;
        }

        Ok(())
    }
}
