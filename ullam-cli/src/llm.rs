use std::{fs::File, path::PathBuf};

use anyhow::Context;
use clap::CommandFactory;
use ullam_common::{APP_DATA_DIR, get_pre_processing_config, llama::serve::binary_location};

use crate::{Args, Model};

pub async fn process(model: Model, ardupilot_file: PathBuf, save: bool) -> anyhow::Result<()> {
    match &model {
        Model::Local { path } if !path.ends_with(".gguf") => {
            Args::command()
                .error(
                    clap::error::ErrorKind::InvalidValue,
                    "Model file should be in `gguf` format",
                )
                .exit();
        }
        _ => (),
    }

    let llama_bin =
        binary_location(&ullam_common::APP_DATA_DIR.join(ullam_common::llama::LLM_DATA_DIR));
    if !llama_bin.exists() {
        tracing::warn!("llama bin not found, downloading new");
        ullam_common::llama::llm_download().await?;
    }

    let dir = APP_DATA_DIR
        .join(format!("analyze_resylts_{}", time::UtcDateTime::now()).replace(' ', "_"));
    if save && !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    let config = get_pre_processing_config()?;

    let logs =
        ullam_parser::LogReader::new(File::open(&ardupilot_file).context("opening logs file")?);

    let processed = ullam_preprocessor::process_with_config(
        logs.into_iter().filter_map(std::result::Result::ok),
        config,
    );

    if save {
        let path = dir.join("preprocessing.json");

        tracing::info!("saving preprocessing results into: {}", path.display());

        let _ = std::fs::write(
            path,
            serde_json::to_string_pretty(&processed).expect("never fails"),
        )
        .inspect_err(|e| tracing::error!(error = ?e, "failed to save intermediate representation"));
    }

    ullam_common::llama::llm_load(model.into()).await?;

    let aggregation_result =
        ullam_common::llama::llm_generate::<ullam_llm::aggregate::FlightAggregation>(
            ullam_llm::AGGREGATE_PROMPT,
            &processed
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .await?;

    if save {
        let path = dir.join("aggregation.json");

        tracing::info!("saving aggregation results into: {}", path.display());

        let _ = std::fs::write(
            path,
            serde_json::to_string_pretty(&aggregation_result).expect("never fails"),
        )
        .inspect_err(|e| tracing::error!(error = ?e, "failed to save intermediate representation"));
    }

    let analyze_result =
        ullam_common::llama::llm_generate::<ullam_llm::aggregate::FlightAggregation>(
            ullam_llm::ANALYZE_PROMPT,
            &aggregation_result.to_string(),
        )
        .await?;

    if save {
        let path = dir.join("analysis.json");

        tracing::info!("saving analysis results into: {}", path.display());

        let _ = std::fs::write(
            path,
            serde_json::to_string_pretty(&analyze_result).expect("never fails"),
        )
        .inspect_err(|e| tracing::error!(error = ?e, "failed to save intermediate representation"));
    }

    println!("{}", analyze_result);

    Ok(())
}
