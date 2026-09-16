use std::{collections::BTreeMap, collections::VecDeque, time::Duration};

use serde::{Deserialize, Serialize};
use ullam_parser::{LogEntry, LogValue};

mod config;

pub use self::config::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub count: usize,
    pub derivative: Option<f64>,
    pub oscillation_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EventCode {
    #[serde(rename = "HIGH_ANGULAR_RATE")]
    HighAngularRate,
    #[serde(rename = "BROWNOUT")]
    Brownout,
    #[serde(rename = "SIGNAL_LOSS")]
    SignalLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventDetails {
    HighAngularRate { field: String, rate: f64 },
    Brownout { field: String, voltage: f64 },
    SignalLoss { field: String, value: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Episode {
    pub ts: f64,
    pub events: Vec<IrRecord>,
    pub root_cause_hypothesis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum IrRecord {
    #[serde(rename = "WINDOW")]
    Window {
        ts: f64,
        msg: String,
        phase: FlightPhase,
        stats: BTreeMap<String, NumericStats>,
        duration_s: f64,
        actuator_saturated: bool,
    },
    #[serde(rename = "EVENT")]
    Event {
        ts: f64,
        msg: String,
        phase: FlightPhase,
        code: EventCode,
        details: EventDetails,
        debounced: bool,
        correlated_events: Vec<EventCode>,
    },
    #[serde(rename = "SNAPSHOT")]
    Snapshot {
        ts: f64,
        msg: String,
        fields: BTreeMap<String, LogValue>,
    },
    #[serde(rename = "EPISODE")]
    Episode(Episode),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PreprocessedLog {
    pub records: Vec<IrRecord>,
}

#[derive(Debug, Clone, Default)]
struct StatsAccumulator {
    count: usize,
    mean: f64,
    m2: f64,
    min: f64,
    max: f64,
}

impl StatsAccumulator {
    fn add(&mut self, value: f64) {
        if self.count == 0 {
            self.min = value;
            self.max = value;
        } else {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
        }
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (value - self.mean);
    }

    fn finish(&self, derivative: Option<f64>) -> NumericStats {
        let stddev = if self.count == 0 {
            0.0
        } else {
            (self.m2 / self.count as f64).sqrt()
        };
        NumericStats {
            min: self.min,
            max: self.max,
            mean: self.mean,
            stddev,
            count: self.count,
            derivative,
            oscillation_index: if self.mean.abs() < f64::EPSILON {
                stddev
            } else {
                stddev / self.mean.abs()
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
struct WindowAccumulator {
    fields: BTreeMap<String, StatsAccumulator>,
    phase: Option<FlightPhase>,
    actuator_saturated: bool,
}

pub fn process(source: impl Iterator<Item = LogEntry>) -> PreprocessedLog {
    process_with_config(source, PreprocessorConfig::default())
}

pub fn process_with_config(
    source: impl Iterator<Item = LogEntry>,
    config: PreprocessorConfig,
) -> PreprocessedLog {
    assert!(config.window_duration > Duration::ZERO);
    let mut windows: BTreeMap<(u128, String), WindowAccumulator> = BTreeMap::new();
    let mut records = Vec::new();
    let mut last_events = BTreeMap::new();
    let mut adaptive_history: BTreeMap<String, VecDeque<f64>> = BTreeMap::new();

    for entry in source {
        if config
            .formatting_messages
            .iter()
            .any(|message| message == &entry.name)
        {
            continue;
        }

        let timestamp = entry.timestamp.unwrap_or_default();
        let timestamp_s = timestamp.as_secs_f64();
        let window_start = timestamp.as_nanos() / config.window_duration.as_nanos();
        let key = (window_start, entry.name.clone());
        let window = windows.entry(key).or_default();
        if let Some(phase) = phase_for_entry(&entry, &config) {
            window.phase = Some(phase);
        }

        for field in &entry.fields {
            if let Some(value) = match &field.value {
                LogValue::I64(value) => Some(*value as f64),
                LogValue::U64(value) => Some(*value as f64),
                LogValue::F64(value)
                | LogValue::Altitude(value)
                | LogValue::ScaledI16(value)
                | LogValue::ScaledI32(value) => Some(*value),
                LogValue::Latitude(value) | LogValue::Longitude(value) => Some(*value as f64),
                LogValue::Time(value) => Some(value.as_secs_f64()),
                LogValue::FlightMode(value) => Some(*value as f64),
                LogValue::Array(_) | LogValue::String(_) => None,
            } {
                window
                    .fields
                    .entry(field.name.clone())
                    .or_default()
                    .add(value);
                if config
                    .actuator_fields
                    .iter()
                    .any(|name| name == &field.name)
                    && actuator_saturated(value, &config)
                {
                    window.actuator_saturated = true;
                }
                detect_events(
                    &mut records,
                    &entry,
                    field,
                    value,
                    timestamp_s,
                    phase_for_entry(&entry, &config).unwrap_or(FlightPhase::Unknown),
                    &config,
                    &mut last_events,
                    &mut adaptive_history,
                );
            }
        }

        if config
            .gps_messages
            .iter()
            .any(|message| message == &entry.name)
        {
            let fields = entry
                .fields
                .iter()
                .map(|field| (field.name.clone(), field.value.clone()))
                .collect();
            records.push(IrRecord::Snapshot {
                ts: timestamp_s,
                msg: entry.name.clone(),
                fields,
            });
        }
    }

    let mut previous_means: BTreeMap<(String, String), f64> = BTreeMap::new();
    for ((window_start, msg), window) in windows {
        let stats = window
            .fields
            .into_iter()
            .map(|(name, values)| {
                let key = (msg.clone(), name.clone());
                let derivative = previous_means.insert(key, values.mean).map(|previous| {
                    (values.mean - previous) / config.window_duration.as_secs_f64()
                });
                (name, values.finish(derivative))
            })
            .collect();
        let duration_s = config.window_duration.as_secs_f64();
        records.push(IrRecord::Window {
            ts: window_start as f64 * duration_s,
            msg,
            phase: window.phase.unwrap_or(FlightPhase::Unknown),
            stats,
            duration_s,
            actuator_saturated: window.actuator_saturated,
        });
    }

    records.sort_by(|left, right| record_timestamp(left).total_cmp(&record_timestamp(right)));
    correlate_events(&mut records, config.event_correlation_window);
    add_episodes(&mut records, config.episode_window);
    if config.relative_timestamps {
        relativize_timestamps(&mut records);
    }
    PreprocessedLog { records }
}

fn detect_events(
    records: &mut Vec<IrRecord>,
    entry: &LogEntry,
    field: &ullam_parser::LogField,
    value: f64,
    timestamp: f64,
    phase: FlightPhase,
    config: &PreprocessorConfig,
    last_events: &mut BTreeMap<EventCode, f64>,
    adaptive_history: &mut BTreeMap<String, VecDeque<f64>>,
) {
    let angular_threshold = adaptive_threshold(
        value,
        config.angular_rate_threshold,
        &field.name,
        config,
        adaptive_history,
    );
    let candidates = [
        (
            config
                .angular_rate_fields
                .iter()
                .any(|name| name == &field.name)
                && value.abs() > angular_threshold,
            EventCode::HighAngularRate,
            EventDetails::HighAngularRate {
                field: field.name.clone(),
                rate: value,
            },
        ),
        (
            config
                .battery_voltage_fields
                .iter()
                .any(|name| name == &field.name)
                && value < config.brownout_voltage_threshold,
            EventCode::Brownout,
            EventDetails::Brownout {
                field: field.name.clone(),
                voltage: value,
            },
        ),
        (
            field.name == config.rc_channel_field && value == config.signal_loss_value,
            EventCode::SignalLoss,
            EventDetails::SignalLoss {
                field: field.name.clone(),
                value,
            },
        ),
    ];
    for (triggered, code, details) in candidates {
        if !triggered {
            continue;
        }
        let debounced = last_events.get(&code).is_some_and(|previous| {
            timestamp - previous < config.event_debounce_window.as_secs_f64()
        });
        if !debounced {
            last_events.insert(code.clone(), timestamp);
            records.push(IrRecord::Event {
                ts: timestamp,
                msg: entry.name.clone(),
                phase: phase.clone(),
                code,
                details,
                debounced: false,
                correlated_events: Vec::new(),
            });
        }
    }
}

fn phase_for_entry(entry: &LogEntry, config: &PreprocessorConfig) -> Option<FlightPhase> {
    entry.fields.iter().find_map(|field| {
        if field.name != config.flight_mode_field {
            return None;
        }
        match field.value {
            LogValue::FlightMode(mode) => Some(
                config
                    .flight_modes
                    .get(&mode)
                    .cloned()
                    .unwrap_or(FlightPhase::Unknown),
            ),
            LogValue::U64(mode) if mode <= u8::MAX as u64 => Some(
                config
                    .flight_modes
                    .get(&(mode as u8))
                    .cloned()
                    .unwrap_or(FlightPhase::Unknown),
            ),
            LogValue::I64(mode) if (0..=u8::MAX as i64).contains(&mode) => Some(
                config
                    .flight_modes
                    .get(&(mode as u8))
                    .cloned()
                    .unwrap_or(FlightPhase::Unknown),
            ),
            _ => None,
        }
    })
}

fn actuator_saturated(value: f64, config: &PreprocessorConfig) -> bool {
    let range = config.actuator_max - config.actuator_min;
    value <= config.actuator_min + range * (1.0 - config.actuator_saturation_ratio)
        || value >= config.actuator_max - range * (1.0 - config.actuator_saturation_ratio)
}

fn adaptive_threshold(
    value: f64,
    fixed: f64,
    field: &str,
    config: &PreprocessorConfig,
    history: &mut BTreeMap<String, VecDeque<f64>>,
) -> f64 {
    let values = history.entry(field.to_owned()).or_default();
    let threshold = if let Some(factor) = config.adaptive_factor {
        if values.is_empty() {
            fixed
        } else {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance =
                values.iter().map(|item| (item - mean).powi(2)).sum::<f64>() / values.len() as f64;
            mean.abs() + factor * variance.sqrt()
        }
    } else {
        fixed
    };
    values.push_back(value);
    while values.len() > config.adaptive_history_windows.max(1) {
        values.pop_front();
    }
    threshold
}

fn correlate_events(records: &mut [IrRecord], window: Duration) {
    let events: Vec<(f64, EventCode)> = records
        .iter()
        .filter_map(|record| match record {
            IrRecord::Event { ts, code, .. } => Some((*ts, code.clone())),
            _ => None,
        })
        .collect();
    for record in records {
        if let IrRecord::Event {
            ts,
            code,
            correlated_events,
            debounced,
            ..
        } = record
        {
            *debounced = true;
            correlated_events.extend(
                events
                    .iter()
                    .filter(|(other_ts, other_code)| {
                        other_code != code && (*other_ts - *ts).abs() <= window.as_secs_f64()
                    })
                    .map(|(_, other_code)| other_code.clone()),
            );
            correlated_events.sort();
            correlated_events.dedup();
        }
    }
}

fn add_episodes(records: &mut Vec<IrRecord>, window: Duration) {
    let events: Vec<IrRecord> = records
        .iter()
        .filter(|record| matches!(record, IrRecord::Event { .. }))
        .cloned()
        .collect();
    if events.len() < 2 {
        return;
    }
    let mut episode_events = vec![events[0].clone()];
    for event in events.into_iter().skip(1) {
        let previous_ts = record_timestamp(episode_events.last().unwrap());
        if record_timestamp(&event) - previous_ts <= window.as_secs_f64() {
            episode_events.push(event);
        } else {
            push_episode(records, &episode_events);
            episode_events = vec![event];
        }
    }
    push_episode(records, &episode_events);
}

fn push_episode(records: &mut Vec<IrRecord>, events: &[IrRecord]) {
    if events.len() < 2 {
        return;
    }
    let ts = record_timestamp(&events[0]);
    let has_signal_loss = events.iter().any(|event| {
        matches!(
            event,
            IrRecord::Event {
                code: EventCode::SignalLoss,
                ..
            }
        )
    });
    let has_brownout = events.iter().any(|event| {
        matches!(
            event,
            IrRecord::Event {
                code: EventCode::Brownout,
                ..
            }
        )
    });
    let root_cause_hypothesis = if has_signal_loss && has_brownout {
        "Signal Loss and Brownout indicate a power-related control failure".to_owned()
    } else {
        "Co-occurring flight anomalies require correlation with telemetry".to_owned()
    };
    records.push(IrRecord::Episode(Episode {
        ts,
        events: events.to_vec(),
        root_cause_hypothesis,
    }));
}

fn relativize_timestamps(records: &mut [IrRecord]) {
    let Some(origin) = records
        .iter()
        .filter_map(|record| match record {
            IrRecord::Event { ts, .. } => Some(*ts),
            _ => None,
        })
        .next()
    else {
        return;
    };
    for record in records {
        shift_timestamp(record, origin);
    }
}

fn shift_timestamp(record: &mut IrRecord, origin: f64) {
    match record {
        IrRecord::Window { ts, .. }
        | IrRecord::Event { ts, .. }
        | IrRecord::Snapshot { ts, .. } => *ts -= origin,
        IrRecord::Episode(Episode { ts, events, .. }) => {
            *ts -= origin;
            for event in events {
                shift_timestamp(event, origin);
            }
        }
    }
}

fn record_timestamp(record: &IrRecord) -> f64 {
    match record {
        IrRecord::Window { ts, .. }
        | IrRecord::Event { ts, .. }
        | IrRecord::Snapshot { ts, .. } => *ts,
        IrRecord::Episode(Episode { ts, .. }) => *ts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(timestamp: u64, name: &str, fields: Vec<(&str, LogValue)>) -> LogEntry {
        LogEntry {
            id: 1,
            name: name.to_owned(),
            timestamp: Some(Duration::from_secs(timestamp)),
            fields: fields
                .into_iter()
                .map(|(name, value)| ullam_parser::LogField {
                    name: name.to_owned(),
                    value,
                })
                .collect(),
        }
    }

    #[test]
    fn aggregates_windows_and_detects_events() {
        let result = process(
            [
                entry(0, "IMU", vec![("GyrY", LogValue::F64(10.0))]),
                entry(0, "RATE", vec![("GyrY", LogValue::F64(200.0))]),
                entry(0, "BAT", vec![("Volt", LogValue::F64(10.0))]),
                entry(0, "RCIN", vec![("C1", LogValue::U64(0))]),
            ]
            .into_iter(),
        );

        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                code: EventCode::HighAngularRate,
                ..
            }
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                code: EventCode::Brownout,
                ..
            }
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                code: EventCode::SignalLoss,
                ..
            }
        )));
        assert!(result.records.iter().any(|record| matches!(
			record,
			IrRecord::Window { stats, .. } if stats.get("GyrY").is_some_and(|value| value.mean == 10.0)
		)));
    }

    #[test]
    fn filters_formatting_messages_but_keeps_string_fields_and_events() {
        let result = process(
            [
                entry(
                    0,
                    "FMTU",
                    vec![
                        ("FmtType", LogValue::U64(1)),
                        ("GyrY", LogValue::F64(300.0)),
                    ],
                ),
                entry(
                    0,
                    "GPS",
                    vec![
                        ("Status", LogValue::String("3D_FIX".to_owned())),
                        ("GyrY", LogValue::F64(200.0)),
                    ],
                ),
            ]
            .into_iter(),
        );

        assert!(!result.records.iter().any(|record| matches!(
            record,
            IrRecord::Window { msg, .. } if msg == "FMTU"
        )));
        assert!(!result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event { msg, .. } if msg == "FMTU"
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                msg,
                code: EventCode::HighAngularRate,
                ..
            } if msg == "GPS"
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Snapshot { fields, .. }
                if fields.get("Status") == Some(&LogValue::String("3D_FIX".to_owned()))
        )));
    }

    #[test]
    fn enriches_windows_and_debounces_repeated_events() {
        let config = PreprocessorConfig::default()
            .window_duration_set(Duration::from_secs(1))
            .actuator_fields_set(vec!["PWM".to_owned()])
            .event_debounce_window_set(Duration::from_secs(2));
        let entries = [
            LogEntry {
                id: 1,
                name: "ATT".to_owned(),
                timestamp: Some(Duration::from_millis(0)),
                fields: vec![
                    ullam_parser::LogField {
                        name: "Mode".to_owned(),
                        value: LogValue::U64(5),
                    },
                    ullam_parser::LogField {
                        name: "GyrY".to_owned(),
                        value: LogValue::F64(160.0),
                    },
                    ullam_parser::LogField {
                        name: "PWM".to_owned(),
                        value: LogValue::F64(100.0),
                    },
                ],
            },
            entry(1, "ATT", vec![("GyrY", LogValue::F64(170.0))]),
        ];
        let result = process_with_config(entries.into_iter(), config);

        assert_eq!(
            result
                .records
                .iter()
                .filter(|record| matches!(record, IrRecord::Event { .. }))
                .count(),
            1
        );
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Window {
                phase: FlightPhase::Loiter,
                actuator_saturated: true,
                stats,
                ..
            } if stats.get("GyrY").is_some()
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Window { stats, .. }
                if stats.get("GyrY").is_some_and(|stats| stats.derivative.is_some())
        )));
    }
}
