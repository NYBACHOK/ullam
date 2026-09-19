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

impl std::fmt::Display for WindowIrRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "W {}", self.msg)?;

        let mut fields: Vec<_> = self.stats.iter().collect();
        fields.sort_unstable_by(|a, b| a.0.cmp(b.0));

        for (field, stats) in fields {
            writeln!(
                f,
                "  {} {:.6}..{:.6} μ={:.6} o={:.6} n={} osc={:.6}",
                field,
                stats.min,
                stats.max,
                stats.mean,
                stats.stddev,
                stats.count,
                stats.oscillation_index,
            )?;
        }

        Ok(())
    }
}

impl std::fmt::Display for SnapshotIrRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "S {}", self.msg)?;

        for (field, value) in &self.fields {
            let value = serde_json::to_string(value).map_err(|_| std::fmt::Error)?;

            writeln!(f, "  {field}={value}")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for PreprocessedLogItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.timestamp.as_secs_f64();
        let end = (self.timestamp + self.duration).as_secs_f64();

        writeln!(f, "CHUNK {start:.3}..{end:.3}")?;

        for window in &self.windows {
            write!(f, "{window}")?;
        }

        for snapshot in &self.snapshots {
            write!(f, "{snapshot}")?;
        }

        Ok(())
    }
}
