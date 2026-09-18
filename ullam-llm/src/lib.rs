pub mod aggregate;
pub mod analysis;

pub fn aggregate_schema() -> schemars::Schema {
    schemars::schema_for!(aggregate::FlightAggregation)
}

pub fn analysis_schema() -> schemars::Schema {
    schemars::schema_for!(analysis::FlightAnalysis)
}
