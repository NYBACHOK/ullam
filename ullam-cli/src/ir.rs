use std::{fs::File, path::PathBuf};

use anyhow::Context;
use ullam_common::get_pre_processing_config;

#[derive(Debug)]
pub enum LogOutput {
    Stdout,
    Json(PathBuf),
    Plain(PathBuf),
}

pub async fn process(ardupilot_file: PathBuf, output: LogOutput) -> anyhow::Result<()> {
    let config = get_pre_processing_config()?;

    let logs =
        ullam_parser::LogReader::new(File::open(&ardupilot_file).context("opening logs file")?);

    let processed = ullam_preprocessor::process_with_config(
        logs.into_iter().filter_map(std::result::Result::ok),
        config,
    );

    match output {
        LogOutput::Stdout => {
            let out = processed
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");

            println!("{out}");
        }
        LogOutput::Json(path) => std::fs::write(
            path,
            serde_json::to_string_pretty(&processed).expect("never fails"),
        )?,
        LogOutput::Plain(path) => {
            let out = processed
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");

            std::fs::write(path, out)?
        }
    }

    Ok(())
}
