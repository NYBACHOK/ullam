use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use ullam_parser::{LogEntry, LogValue};

mod config;
mod types;
mod utils;

use crate::utils::StatsAccumulator;

pub use self::{config::*, types::*};

pub fn process(source: impl Iterator<Item = LogEntry>) -> Vec<PreprocessedLogItem> {
    process_with_config(source, PreprocessorConfig::default())
}

pub fn process_with_config(
    source: impl IntoIterator<Item = LogEntry>,
    PreprocessorConfig {
        window_duration,
        messages_to_ignore,
        snapshoting_fields,
        ignore_fields,
        msg_with_ignored_fields,
    }: PreprocessorConfig,
) -> Vec<PreprocessedLogItem> {
    assert!(
        window_duration != Duration::ZERO,
        "config `window_duration` can't be ZERO"
    );

    utils::chunks_by_duration(source, window_duration, messages_to_ignore)
        .map(|this| {
            process_chunk(
                this,
                window_duration,
                &snapshoting_fields,
                &ignore_fields,
                &msg_with_ignored_fields,
            )
        })
        .collect()
}

fn process_chunk(
    source: Vec<LogEntry>,
    duration: Duration,
    snapshoting_fields: &HashSet<String>,
    ignore_fields: &HashSet<String>,
    msg_with_ignored_fields: &HashMap<String, HashSet<String>>,
) -> PreprocessedLogItem {
    let mut timestamp = None;

    let mut ir_snapshoted_fields = HashMap::<String, Vec<(String, LogValue)>>::new();
    let mut ir_stats_accumulators = HashMap::<String, HashMap<String, StatsAccumulator>>::new();

    source.into_iter().for_each(|msg| {
        if timestamp.is_none() {
            timestamp = Some(msg.timestamp);
        }

        let fields_to_ignore = msg_with_ignored_fields.get(&msg.name);

        for field in msg.fields {
            if ignore_fields.contains(&field.name)
                || fields_to_ignore.is_some_and(|this| this.contains(&field.name))
            {
                continue;
            }

            if snapshoting_fields.contains(&field.name) {
                let snapshoted_fields = if let Some(val) = ir_snapshoted_fields.get_mut(&msg.name) {
                    val
                } else {
                    ir_snapshoted_fields.insert(msg.name.clone(), Vec::new());

                    ir_snapshoted_fields
                        .get_mut(&msg.name)
                        .expect("we inserted above")
                };

                let field_to_snapshot = (field.name, field.value);

                if !snapshoted_fields.contains(&field_to_snapshot) {
                    snapshoted_fields.push(field_to_snapshot);
                }

                continue;
            }

            let fields_and_accm = if let Some(val) = ir_stats_accumulators.get_mut(&msg.name) {
                val
            } else {
                ir_stats_accumulators.insert(msg.name.clone(), HashMap::new());

                ir_stats_accumulators
                    .get_mut(&msg.name)
                    .expect("we inserted above")
            };

            let stats = if let Some(stats) = fields_and_accm.get_mut(&field.name) {
                stats
            } else {
                fields_and_accm.insert(field.name.clone(), StatsAccumulator::default());

                fields_and_accm
                    .get_mut(&field.name)
                    .expect("inserted above")
            };

            let value = field.value.try_into_float().unwrap_or_else(|| {
                panic!(
                    "tried to write string or array as number for {}",
                    field.name
                )
            });

            stats.add(value);
        }
    });

    PreprocessedLogItem {
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
    }
}
