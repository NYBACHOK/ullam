use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        ::std::collections::HashMap::from([
            $(
                ($key.to_string(), $value),
            )*
        ])
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PreprocessorConfig {
    pub window_duration: Duration,
    /// Messages and fields that are relevant to flight analysis.
    pub messages: HashMap<String, MessageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageConfig {
    pub fields: HashMap<String, FieldConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    pub mode: FieldMode,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum FieldMode {
    #[default]
    Ignore,
    Stats,
    Snapshot,
    Event,
    Counter,
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        let messages = hashmap! {
            "ATT" => hashmap! {
                "Roll"      => FieldMode::Stats,
                "Pitch"     => FieldMode::Stats,
                "Yaw"       => FieldMode::Stats,
                "DesRoll"   => FieldMode::Stats,
                "DesPitch"  => FieldMode::Stats,
                "DesYaw"    => FieldMode::Stats,
            },

            "VIBE" => hashmap! {
                "VibeX" => FieldMode::Stats,
                "VibeY" => FieldMode::Stats,
                "VibeZ" => FieldMode::Stats,
                "Clip"  => FieldMode::Stats,
            },

            "MODE" => hashmap! {
                "Mode"    => FieldMode::Snapshot,
                "ModeNum" => FieldMode::Snapshot,
                "Rsn"     => FieldMode::Snapshot,
            },

            "RCOU" => hashmap! {
                "C1" => FieldMode::Stats,
                "C2" => FieldMode::Stats,
                "C3" => FieldMode::Stats,
                "C4" => FieldMode::Stats,
            },

            "BAT" => hashmap! {
                "Volt"    => FieldMode::Stats,
                "Curr"    => FieldMode::Stats,
                "CurrTot" => FieldMode::Stats,
                "RemPct"  => FieldMode::Stats,
            },

            "MSG" => hashmap! {
                "Message"   => FieldMode::Event,
            },
        }
        .into_iter()
        .map(|(msg, fields)| {
            (
                msg,
                MessageConfig {
                    fields: fields
                        .into_iter()
                        .map(|(field, mode)| (field, FieldConfig { mode }))
                        .collect(),
                },
            )
        })
        .collect();

        Self {
            window_duration: Duration::from_secs(20),
            messages,
        }
    }
}

impl PreprocessorConfig {
    #[must_use]
    pub fn window_duration_set(mut self, window_duration: Duration) -> Self {
        self.window_duration = window_duration;
        self
    }
}
