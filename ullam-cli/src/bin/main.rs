use clap::CommandFactory;
use ullam_cli::{
    Args,
    ir::{self, LogOutput},
    llm,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        log_level,
        ardupilot_file,
        command,
        save,
        vehicle_type,
        ..
    } = <Args as clap::Parser>::parse();

    let ardupilot_file = match ardupilot_file {
        Some(file) if file.exists() => file,
        _ => Args::command()
            .error(
                clap::error::ErrorKind::InvalidValue,
                "Failed to find ardupilot file",
            )
            .exit(),
    };

    setup_logger(log_level);

    match command {
        ullam_cli::Subcommand::Llm(model) => {
            llm::process(model, ardupilot_file, vehicle_type, save).await?;
        }
        ullam_cli::Subcommand::IR { output } => {
            let output = match output {
                Some(path) if path.extension().unwrap_or_default() == "json" => {
                    LogOutput::Json(path)
                }
                Some(path) => LogOutput::Plain(path),
                None => LogOutput::Stdout,
            };

            ir::process(ardupilot_file, output, vehicle_type).await?;
        }
    }

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
        .add_directive("reqwest=warn".parse().unwrap())
        .add_directive("async_openai=error".parse().unwrap())
        .add_directive("hyper_util=warn".parse().unwrap());

    let registry = tracing_subscriber::registry().with(filter);

    registry.with(tracing_subscriber::fmt::layer()).init();
}
