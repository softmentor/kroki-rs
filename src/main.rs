use clap::{Parser, Subcommand};
use kroki_rs::{cli, config::Config, server};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "kroki-rs")]
#[command(version, about = "Rust port of Kroki", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server
    Serve {
        #[arg(short, long, default_value_t = 8000)]
        port: u16,
    },
    /// Convert a diagram file
    Convert {
        #[arg(short, long)]
        type_: String,
        #[arg(short, long)]
        format: String,
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = Cli::parse();
    let config = Config::load(args.config)?;

    match args.command {
        Commands::Serve { port } => {
            info!("Starting server on port {}", port);
            server::run(port, config).await?;
        }
        Commands::Convert {
            type_,
            format,
            input,
        } => {
            cli::convert(type_, format, input, config).await?;
        }
    }

    Ok(())
}
