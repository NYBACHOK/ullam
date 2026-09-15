use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PreprocessorConfig {
    pub window_duration: Duration,
    pub angular_rate_threshold: f64,
    pub brownout_voltage_threshold: f64,
    pub signal_loss_value: f64,
    pub angular_rate_fields: Vec<String>,
    pub battery_voltage_fields: Vec<String>,
    pub rc_channel_field: String,
    pub gps_messages: Vec<String>,
    pub formatting_messages: Vec<String>,
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        Self {
            window_duration: Duration::from_secs(1),
            angular_rate_threshold: 150.0,
            brownout_voltage_threshold: 10.5,
            signal_loss_value: 0.0,
            angular_rate_fields: ["pitch_rate", "PitchRate", "PRate", "GyrY"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            battery_voltage_fields: ["battery_voltage", "BatteryVoltage", "Volt", "Vcc"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            rc_channel_field: "C1".to_owned(),
            gps_messages: ["GPS".to_owned(), "GPA".to_owned(), "GRAW".to_owned()].to_vec(),
            formatting_messages: ["FMT", "FMTU", "FMU", "UNIT", "MULT"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }
}

impl PreprocessorConfig {
    pub fn window_duration_set(mut self, window_duration: Duration) -> Self {
        self.window_duration = window_duration;
        self
    }

    pub fn angular_rate_threshold_set(mut self, angular_rate_threshold: f64) -> Self {
        self.angular_rate_threshold = angular_rate_threshold;
        self
    }

    pub fn brownout_voltage_threshold_set(mut self, brownout_voltage_threshold: f64) -> Self {
        self.brownout_voltage_threshold = brownout_voltage_threshold;
        self
    }

    pub fn signal_loss_value_set(mut self, signal_loss_value: f64) -> Self {
        self.signal_loss_value = signal_loss_value;
        self
    }

    pub fn angular_rate_fields_set(mut self, angular_rate_fields: Vec<String>) -> Self {
        self.angular_rate_fields = angular_rate_fields;
        self
    }

    pub fn battery_voltage_fields_set(mut self, battery_voltage_fields: Vec<String>) -> Self {
        self.battery_voltage_fields = battery_voltage_fields;
        self
    }

    pub fn rc_channel_field_set(mut self, rc_channel_field: String) -> Self {
        self.rc_channel_field = rc_channel_field;
        self
    }

    pub fn gps_messages_set(mut self, gps_messages: Vec<String>) -> Self {
        self.gps_messages = gps_messages;
        self
    }

    pub fn formatting_messages_set(mut self, formatting_messages: Vec<String>) -> Self {
        self.formatting_messages = formatting_messages;
        self
    }
}
