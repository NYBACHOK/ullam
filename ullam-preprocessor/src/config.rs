use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PreprocessorConfig {
    pub window_duration: Duration,
    /// Messages that should be ignored
    pub messages_to_ignore: HashSet<String>,
    /// Fields that should be snapshoted rather than calculate stats
    pub snapshoting_fields: HashSet<String>,
    /// Fields to ignore
    pub ignore_fields: HashSet<String>,
    /// Messages that have fields which should be ignored
    pub msg_with_ignored_fields: HashMap<String, HashSet<String>>,
    /// Message with mavlink messages
    pub mavlink_messages: MavlinkMessagesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MavlinkMessagesConfig {
    pub name: String,
    pub field_name: String,
}

impl Default for MavlinkMessagesConfig {
    fn default() -> Self {
        Self {
            name: "MSG".to_owned(),
            field_name: "Message".to_owned(),
        }
    }
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        Self {
            window_duration: Duration::from_secs(20),
            messages_to_ignore: [
                "FMT", "FMTU", "FMU", "UNIT", "MULT", "FILE", "VER", "GPS", "PARM", "STAK", "PIDP",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            snapshoting_fields: ["Mode", "Name", "Value", "ModeNum", "Rsn"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            ignore_fields: ["TimeUS", "Default"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            msg_with_ignored_fields: [("PIDP", vec!["I"])]
                .into_iter()
                .map(|(key, value)| {
                    (
                        key.to_owned(),
                        value
                            .into_iter()
                            .map(|this: &str| this.to_owned())
                            .collect(),
                    )
                })
                .collect(),
            mavlink_messages: Default::default(),
        }
    }
}

impl PreprocessorConfig {
    #[must_use]
    pub fn window_duration_set(mut self, window_duration: Duration) -> Self {
        self.window_duration = window_duration;
        self
    }

    pub fn messages_to_ignore(
        mut self,
        messages_to_ignore: impl IntoIterator<Item = String>,
    ) -> Self {
        self.messages_to_ignore = messages_to_ignore.into_iter().collect();
        self
    }

    pub fn snapshoting_fields(
        mut self,
        snapshoting_fields: impl IntoIterator<Item = String>,
    ) -> Self {
        self.snapshoting_fields = snapshoting_fields.into_iter().collect();
        self
    }

    pub fn ignore_fields(mut self, ignore_fields: impl IntoIterator<Item = String>) -> Self {
        self.ignore_fields = ignore_fields.into_iter().collect();
        self
    }
}
