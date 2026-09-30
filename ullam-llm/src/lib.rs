pub mod aggregate;
pub mod analysis;

pub const AGGREGATE_PROMPT: &str = include_str!("../../assets/aggregate_prompt");
pub const ANALYZE_PROMPT: &str = include_str!("../../assets/analyze_prompt");

fn skip_serializing_if_f64(val: &f64) -> bool {
    !val.is_normal() || *val == 0.0
}
