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

    #[arg(long, value_name = "DIR")]
    cache_dir: Option<PathBuf>,
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
        /// Optional: WebP quality ('lossless', 'high', 'medium', 'low', or 0-100)
        #[arg(long)]
        webp_quality: Option<String>,
    },
    /// Convert all diagrams in a directory
    Batch {
        #[arg(short, long, default_value = "svg")]
        format: String,
        #[arg(value_name = "DIR")]
        input: PathBuf,
        /// Optional: Force a specific diagram type for all files
        #[arg(short, long)]
        type_: Option<String>,
        /// Optional: Output directory (defaults to input dir)
        #[arg(short, long)]
        out_dir: Option<PathBuf>,
        /// Optional: WebP quality ('lossless', 'high', 'medium', 'low', or 0-100)
        #[arg(long)]
        webp_quality: Option<String>,
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
            webp_quality,
        } => {
            let mut config = config;
            if let Some(wq) = webp_quality {
                config.webp.quality = wq;
            }
            cli::convert(type_, format, input, config, args.cache_dir).await?;
        }
        Commands::Batch {
            format,
            input,
            type_,
            out_dir,
            webp_quality,
        } => {
            let mut config = config;
            if let Some(wq) = webp_quality {
                config.webp.quality = wq;
            }
            cli::batch(format, input, type_, out_dir, config, args.cache_dir).await?;
        }
    }

    Ok(())
}
