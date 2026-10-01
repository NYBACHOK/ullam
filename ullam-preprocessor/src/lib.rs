use std::{collections::HashMap, time::Duration};

use ullam_parser::{LogEntry, LogValue};

mod config;
mod types;
mod utils;

use crate::utils::{HashMapExt, StatsAccumulator};

pub use self::{config::*, types::*};

pub fn process(source: impl Iterator<Item = LogEntry>) -> Vec<PreprocessedLogItem> {
    process_with_config(source, PreprocessorConfig::default())
}

pub fn process_with_config(
    source: impl IntoIterator<Item = LogEntry>,
    PreprocessorConfig {
        window_duration,
        messages,
    }: PreprocessorConfig,
) -> Vec<PreprocessedLogItem> {
    assert!(
        window_duration != Duration::ZERO,
        "config `window_duration` can't be ZERO"
    );

    utils::chunks_by_duration(source, window_duration)
        .filter_map(|this| process_chunk(this, window_duration, &messages))
        .collect()
}

fn process_chunk(
    source: Vec<LogEntry>,
    duration: Duration,
    messages: &HashMap<String, MessageConfig>,
) -> Option<PreprocessedLogItem> {
    let mut timestamp = None;

    let mut ir_events = Vec::new();
    let mut ir_snapshoted_fields = HashMap::<String, Vec<(String, LogValue)>>::new();
    let mut ir_stats_accumulators = HashMap::<String, HashMap<String, StatsAccumulator>>::new();
    let mut ir_count_msg = HashMap::<String, u64>::new();

    source
        .into_iter()
        .filter_map(|this| match messages.get(&this.name) {
            Some(cfg) => Some((this, cfg)),
            None => None,
        })
        .for_each(|(log_entry, msg_cfg)| {
            if timestamp.is_none() && log_entry.timestamp.is_some() {
                timestamp = Some(log_entry.timestamp);
            }

            log_entry
                .fields
                .into_iter()
                .filter_map(|this| match msg_cfg.fields.get(&this.name) {
                    Some(field_cfg) => Some((this, field_cfg)),
                    None => None,
                })
                .for_each(|(log_field, field_cfg)| match field_cfg.mode {
                    FieldMode::Ignore => return,
                    FieldMode::Event => {
                        if let Some(value) = log_field.value.try_into_string() {
                            ir_events.push(value);
                        }
                    }
                    FieldMode::Snapshot => {
                        let snapshoted_fields =
                            ir_snapshoted_fields.get_mut_or_insert_default(&log_entry.name);

                        let field_to_snapshot = (log_field.name, log_field.value);

                        if !snapshoted_fields.contains(&field_to_snapshot) {
                            snapshoted_fields.push(field_to_snapshot);
                        }
                    }
                    FieldMode::Stats => {
                        let fields_stats =
                            ir_stats_accumulators.get_mut_or_insert_default(&log_entry.name);

                        let stats = fields_stats.get_mut_or_insert_default(&log_field.name);

                        let value = log_field.value.try_into_float().unwrap_or_else(|| {
                            panic!(
                                "tried to write string or array as number for {}",
                                log_field.name
                            )
                        });

                        stats.add(value);
                    }
                    FieldMode::Counter => {
                        let count = ir_count_msg.get_mut_or_insert_default(&log_entry.name);
                        *count += 1;
                    }
                });
        });

    if ir_events.is_empty()
        && ir_count_msg.is_empty()
        && ir_snapshoted_fields.is_empty()
        && ir_stats_accumulators.is_empty()
    {
        return None;
    }

    Some(PreprocessedLogItem {
        timestamp: timestamp.flatten().unwrap_or_default(),
        duration,
        windows: ir_stats_accumulators
            .into_iter()
            .map(|(msg_name, windows)| WindowIrRecord {
                msg: msg_name,
                stats: windows
                    .into_iter()
                    .map(|(field_name, stats)| (field_name, stats.finish()))
                    .collect(),
            })
            .collect(),
        snapshots: ir_snapshoted_fields
            .into_iter()
            .map(|(msg_name, fields)| SnapshotIrRecord {
                msg: msg_name,
                fields,
            })
            .collect(),
        messages: ir_events,
        count: ir_count_msg,
    })
}
