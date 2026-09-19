# Ullam

Ullam is a workflow for simplifying ArduPilot flight-log analysis by turning raw telemetry into a form that is easier for an LLM to reason about.

The project is built around a two-stage analysis flow:

1. Raw ArduPilot logs are parsed and preprocessed into a cleaner, structured representation.
2. That representation is passed through an LLM in two stages: first to aggregate flight information into meaningful summaries, then to analyze the aggregated results for the overall flight outcome and key events.

## What it does

- Reads ArduPilot BIN logs
- Normalizes the log stream into a structured intermediate representation
- Uses a first LLM pass to summarize flight segments and flight metadata
- Uses a second LLM pass to reason over the whole mission and produce the final analysis
- Optionally saves preprocessing, aggregation, and final analysis artifacts for inspection

## Purpose

The goal is to make flight-log analysis more practical and more consistent by moving the heavy lifting into preprocessing and staged LLM reasoning instead of asking a single model to interpret raw logs directly or manual extraction of prefered fields.

## Prerequisites

Install Rust on Linux/MacOS with:

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

If you need install on Windows visit [this](https://rust-lang.org/tools/install/) page.

## Quick start

Run analysis against a log file with a downloaded model from HuggingFace:

```bash
cargo run --release -- --log-level info --ardupilot-file /path/to/flight.BIN hf-model --save
```

Or use a local GGUF model:

```bash
cargo run --release -- --log-level info --ardupilot-file /path/to/flight.BIN local /path/to/model.gguf --save
```

The `--save` flag stores intermediate JSON outputs alongside the analysis results for easier debugging and review.
