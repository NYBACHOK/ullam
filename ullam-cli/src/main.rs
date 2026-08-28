use std::path::PathBuf;

use anyhow::Context;

#[derive(clap_derive::Parser)]
#[non_exhaustive]
struct Args {
    /// The path to the model
    #[command(subcommand)]
    model: Model,
    #[arg(long, global = true, required = false, default_value_t = default_log_level())]
    log_level: tracing::Level,
    #[arg(long, global = true, required = false)]
    ardupilot_file: Option<PathBuf>,
}

#[derive(clap_derive::Subcommand, Debug, Clone)]
enum Model {
    /// Use an already downloaded model
    Local {
        /// The path to the model.
        #[arg(required = true)]
        path: PathBuf,
    },
    /// Download a model from huggingface (or use a cached version)
    #[clap(name = "hf-model")]
    HuggingFace {
        /// the model name.
        #[arg(required = false, default_value_t = String::from("Ministral-3b-instruct.Q2_K.gguf"))]
        model: String,
        /// owner of the repo
        #[arg(required = false, default_value_t = String::from("QuantFactory"))]
        owner: String,
        /// the repo containing the model.
        #[arg(required = false, default_value_t = String::from("Ministral-3b-instruct-GGUF"))]
        repo: String,
    },
}

impl From<Model> for ullam_common::llama::Model {
    fn from(value: Model) -> Self {
        match value {
            Model::Local { path } => Self::Local(path),
            Model::HuggingFace { model, owner, repo } => Self::HuggingFace { model, owner, repo },
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        log_level,
        model: _,
        ardupilot_file,
        ..
    } = <Args as clap::Parser>::parse();

    setup_logger(log_level);

    if let Some(path) = ardupilot_file {
        parse_ardupilot_log(&path)?;
    }

    Ok(())
}

fn parse_ardupilot_log(path: &PathBuf) -> anyhow::Result<()> {
    let file = ardupilot_binlog::File::open(path)
        .with_context(|| format!("failed to open ArduPilot log {}", path.display()))?;
    let entries = file
        .entries()
        .with_context(|| format!("failed to read ArduPilot log {}", path.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| {
            format!(
                "failed to parse an entry in ArduPilot log {}",
                path.display()
            )
        })?;
        let fields = entry
            .fields()
            .map(|(name, value)| -> anyhow::Result<_> {
                Ok((name.to_owned(), field_value_to_json(value)?))
            })
            .collect::<Result<serde_json::Map<_, _>, _>>()?;
        let typed =
            serde_json::from_value::<ullam_ardupilot_log_types::Message>(serde_json::json!({
                "message": entry.name,
                "data": fields,
            }))
            .with_context(|| "failed to deserialize ArduPilot entry into generated message type")?;

        tracing::debug!(
            msg_type = entry.msg_type,
            timestamp_usec = ?entry.timestamp_usec,
            message = ?typed,
            "parsed ArduPilot message"
        );
    }

    Ok(())
}

fn field_value_to_json(value: &ardupilot_binlog::FieldValue) -> anyhow::Result<serde_json::Value> {
    let value = match value {
        ardupilot_binlog::FieldValue::Int(value) => serde_json::json!(value),
        ardupilot_binlog::FieldValue::Uint(value) => serde_json::json!(value),
        ardupilot_binlog::FieldValue::Float(value) => serde_json::json!(value),
        ardupilot_binlog::FieldValue::String(value) => {
            serde_json::Value::Array(value.bytes().map(serde_json::Value::from).collect())
        }
        ardupilot_binlog::FieldValue::Array(value) => serde_json::json!(value),
    };
    Ok(value)
}

fn setup_logger(log_level: tracing::Level) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env()
        .expect("default level is set")
        .add_directive("xet_client=warn".parse().unwrap())
        .add_directive("xet_data=warn".parse().unwrap())
        .add_directive("xet=warn".parse().unwrap())
        .add_directive("hyper_util=warn".parse().unwrap());

    let registry = tracing_subscriber::registry().with(filter);

    registry.with(tracing_subscriber::fmt::layer()).init()
}

fn default_log_level() -> tracing::Level {
    if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    }
}
