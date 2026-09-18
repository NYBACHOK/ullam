use std::{collections::HashSet, time::Duration};
use ullam_parser::LogEntry;

use crate::NumericStats;

pub fn chunks_by_duration<I>(
    iter: I,
    window: Duration,
    msg_to_ignore: HashSet<String>,
) -> impl Iterator<Item = Vec<LogEntry>>
where
    I: IntoIterator<Item = LogEntry>,
{
    let mut iter = iter.into_iter();
    let mut chunk = Vec::new();
    let mut start_ts = None;

    std::iter::from_fn(move || {
        loop {
            match iter.next() {
                Some(item) if msg_to_ignore.contains(&item.name) => continue,
                Some(item) => {
                    let ts = item
                        .timestamp
                        .expect("all messages without timestamp should be filtered out");
                    match start_ts {
                        None => {
                            start_ts = Some(ts);
                            chunk.push(item);
                        }
                        Some(start) if ts.saturating_sub(start) <= window => {
                            chunk.push(item);
                        }
                        Some(_) => {
                            let result = std::mem::take(&mut chunk);
                            start_ts = Some(ts);
                            chunk.push(item);
                            return Some(result);
                        }
                    }
                }
                None => {
                    if chunk.is_empty() {
                        return None;
                    }
                    start_ts = None;
                    return Some(std::mem::take(&mut chunk));
                }
            }
        }
    })
}

#[derive(Debug, Clone, Default)]
pub struct StatsAccumulator {
    count: usize,
    mean: f64,
    m2: f64,
    min: f64,
    max: f64,
}

impl StatsAccumulator {
    pub fn add(&mut self, value: f64) {
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

    pub fn finish(&self) -> NumericStats {
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
            oscillation_index: if self.mean.abs() < f64::EPSILON {
                stddev
            } else {
                stddev / self.mean.abs()
            },
        }
    }
}
