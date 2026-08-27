use std::path::PathBuf;

#[derive(clap_derive::Parser)]
#[non_exhaustive]
struct Args {
    /// The path to the model
    #[command(subcommand)]
    model: Model,
    #[arg(long, global = true, required = false, default_value_t = default_log_level())]
    log_level: tracing::Level,
    // #[arg(long, global = true, required = true)]
    // ardupilot_file: PathBuf,
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
        log_level, model, ..
    } = <Args as clap::Parser>::parse();

    setup_logger(log_level);

    Ok(())
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
