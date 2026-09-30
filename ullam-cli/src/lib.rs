use std::path::PathBuf;

pub mod llm;
pub mod ir;

#[derive(clap_derive::Parser)]
#[non_exhaustive]
pub struct Args {
    /// The path to the model
    #[command(subcommand)]
    pub command: Subcommand,
    /// Save analyzing result into app dir
    #[arg(short, long, required = false, global = true, default_value_t = false)]
    pub save: bool,
    #[arg(long, global = true, required = false, default_value_t = default_log_level())]
    pub log_level: tracing::Level,
    #[arg(short = 'i', long, global = true, required = false)]
    pub ardupilot_file: Option<PathBuf>,
}

fn default_log_level() -> tracing::Level {
    if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    }
}

#[derive(clap_derive::Subcommand)]
pub enum Subcommand {
    /// Full processing of BIN file with analysis by LLM of choice
    #[clap(name = "llm")]
    #[command(subcommand)]
    Llm(Model),
    /// Process BIN file and output only results of pre-processing stage to manual analysis
    #[clap(name = "ir")]
    IR {
        /// File for outputing result, if not specified will print result to stdout
        #[arg(short, long, required = false)]
        output: Option<PathBuf>,
    },
}

#[derive(clap_derive::Subcommand, Debug, Clone)]
pub enum Model {
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
        #[arg(required = false, default_value_t = String::from("Ministral-3-14B-Reasoning-2512-Q4_K_M.gguf"))]
        model: String,
        /// owner of the repo
        #[arg(required = false, default_value_t = String::from("mistralai"))]
        owner: String,
        /// the repo containing the model.
        #[arg(required = false, default_value_t = String::from("Ministral-3-14B-Reasoning-2512-GGUF"))]
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
