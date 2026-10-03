use std::{collections::HashMap, hash::Hash, time::Duration};
use ullam_parser::LogEntry;

use crate::NumericStats;

pub fn chunks_by_duration<I>(iter: I, window: Duration) -> impl Iterator<Item = Vec<LogEntry>>
where
    I: IntoIterator<Item = LogEntry>,
{
    let mut iter = iter.into_iter();
    let mut chunk = Vec::new();
    let mut start_ts = None;

    std::iter::from_fn(move || {
        loop {
            if let Some(item) = iter.next() {
                let ts = if let Some(ts) = item.timestamp {
                    ts
                } else {
                    continue;
                };

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
            } else {
                if chunk.is_empty() {
                    return None;
                }
                start_ts = None;
                return Some(std::mem::take(&mut chunk));
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
        }
    }
}

pub trait HashMapExt<K, V> {
    fn get_mut_or_insert_default(&mut self, key: &K) -> &mut V;
}

impl<K: Clone + Eq + Hash, V: Default> HashMapExt<K, V> for HashMap<K, V> {
    fn get_mut_or_insert_default(&mut self, key: &K) -> &mut V {
        if !self.contains_key(key) {
            self.insert(key.clone(), Default::default());
        }

        self.get_mut(key).expect("we inserted above")
    }
}
