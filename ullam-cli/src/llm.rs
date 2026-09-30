use std::{fs::File, io::Write, path::PathBuf};

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

    let start_index = select_start_index(&processed)?;
    ullam_common::llama::llm_load(model.into()).await?;

    let selected_processed = &processed[start_index..];
    let mut aggregation_results = Vec::with_capacity(selected_processed.len());

    for item in selected_processed {
        let aggregation_result = ullam_common::llama::llm_generate::<
            ullam_llm::aggregate::FlightAggregation,
        >(ullam_llm::AGGREGATE_PROMPT, &item.to_string())
        .await;

        match aggregation_result {
            Ok(v) => aggregation_results.push(v),
            Err(e) => tracing::error!(error = ?e, "failed to process chunk, ignoring it"),
        }
    }

    if save {
        let path = dir.join("aggregation.json");

        tracing::info!("saving aggregation results into: {}", path.display());

        let _ = std::fs::write(
            path,
            serde_json::to_string_pretty(&aggregation_results).expect("never fails"),
        )
        .inspect_err(|e| tracing::error!(error = ?e, "failed to save intermediate representation"));
    }

    let analyze_result =
        ullam_common::llama::llm_generate::<ullam_llm::aggregate::FlightAggregation>(
            ullam_llm::ANALYZE_PROMPT,
            &aggregation_results
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
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

fn select_start_index(
    processed: &[ullam_preprocessor::PreprocessedLogItem],
) -> anyhow::Result<usize> {
    if processed.is_empty() {
        anyhow::bail!("no chunks were produced from the log file");
    }

    let start = processed
        .first()
        .expect("checked that processed is not empty");
    let end = processed
        .last()
        .expect("checked that processed is not empty");
    let end_timestamp = end.timestamp + end.duration;

    println!(
        "Processed {} chunks covering {:.3}s to {:.3}s.",
        processed.len(),
        start.timestamp.as_secs_f64(),
        end_timestamp.as_secs_f64(),
    );

    loop {
        print!(
            "How many chunks should be skipped before analysis? [0-{}]: ",
            processed.len() - 1
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(index) if index < processed.len() => return Ok(index),
            _ => println!("Please enter a number from 0 to {}.", processed.len() - 1),
        }
    }
}
