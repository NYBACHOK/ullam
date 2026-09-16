use std::{collections::BTreeMap, time::Duration};

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
    pub flight_mode_field: String,
    pub flight_modes: BTreeMap<u8, FlightPhase>,
    pub event_debounce_window: Duration,
    pub event_correlation_window: Duration,
    pub adaptive_factor: Option<f64>,
    pub adaptive_history_windows: usize,
    pub actuator_fields: Vec<String>,
    pub actuator_min: f64,
    pub actuator_max: f64,
    pub actuator_saturation_ratio: f64,
    pub relative_timestamps: bool,
    pub episode_window: Duration,
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
            flight_mode_field: "Mode".to_owned(),
            flight_modes: BTreeMap::from([
                (0, FlightPhase::Stabilize),
                (1, FlightPhase::Stabilize),
                (5, FlightPhase::Loiter),
                (6, FlightPhase::Rtl),
                (14, FlightPhase::Flip),
            ]),
            event_debounce_window: Duration::from_millis(200),
            event_correlation_window: Duration::from_millis(50),
            adaptive_factor: None,
            adaptive_history_windows: 10,
            actuator_fields: Vec::new(),
            actuator_min: 0.0,
            actuator_max: 100.0,
            actuator_saturation_ratio: 0.95,
            relative_timestamps: false,
            episode_window: Duration::from_secs(2),
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

    pub fn flight_mode_field_set(mut self, flight_mode_field: String) -> Self {
        self.flight_mode_field = flight_mode_field;
        self
    }

    pub fn flight_modes_set(mut self, flight_modes: BTreeMap<u8, FlightPhase>) -> Self {
        self.flight_modes = flight_modes;
        self
    }

    pub fn event_debounce_window_set(mut self, event_debounce_window: Duration) -> Self {
        self.event_debounce_window = event_debounce_window;
        self
    }

    pub fn event_correlation_window_set(mut self, event_correlation_window: Duration) -> Self {
        self.event_correlation_window = event_correlation_window;
        self
    }

    pub fn adaptive_factor_set(mut self, adaptive_factor: Option<f64>) -> Self {
        self.adaptive_factor = adaptive_factor;
        self
    }

    pub fn adaptive_history_windows_set(mut self, adaptive_history_windows: usize) -> Self {
        self.adaptive_history_windows = adaptive_history_windows;
        self
    }

    pub fn actuator_fields_set(mut self, actuator_fields: Vec<String>) -> Self {
        self.actuator_fields = actuator_fields;
        self
    }

    pub fn actuator_range_set(mut self, actuator_min: f64, actuator_max: f64) -> Self {
        self.actuator_min = actuator_min;
        self.actuator_max = actuator_max;
        self
    }

    pub fn relative_timestamps_set(mut self, relative_timestamps: bool) -> Self {
        self.relative_timestamps = relative_timestamps;
        self
    }

    pub fn episode_window_set(mut self, episode_window: Duration) -> Self {
        self.episode_window = episode_window;
        self
    }
}
