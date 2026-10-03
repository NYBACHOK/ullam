use std::{fs::File, path::PathBuf};

use anyhow::Context;
use ullam_common::get_pre_processing_config;
use ullam_preprocessor::VenicleType;

#[derive(Debug)]
pub enum LogOutput {
    Stdout,
    Json(PathBuf),
    Plain(PathBuf),
}

pub async fn process(
    ardupilot_file: PathBuf,
    output: LogOutput,
    venicle_type: VenicleType,
) -> anyhow::Result<()> {
    let config = get_pre_processing_config()?;

    let logs =
        ullam_parser::LogReader::new(File::open(&ardupilot_file).context("opening logs file")?);

    let processed = ullam_preprocessor::process_with_config(
        logs.into_iter().filter_map(std::result::Result::ok),
        config,
        venicle_type,
    );

    match output {
        LogOutput::Stdout => {
            println!("{processed}");
        }
        LogOutput::Json(path) => std::fs::write(
            path,
            serde_json::to_string_pretty(&processed).expect("never fails"),
        )?,
        LogOutput::Plain(path) => std::fs::write(path, processed.to_string())?,
    }

    Ok(())
}
