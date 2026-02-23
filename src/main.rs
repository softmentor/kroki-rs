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
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Convert a diagram file
    Convert {
        #[arg(short, long)]
        type_: String,
        #[arg(short, long)]
        format: String,
        #[arg(value_name = "FILE")]
        input: PathBuf,
        /// Optional: External font URLs to download and rasterize
        #[arg(long, value_name = "URL")]
        font: Vec<String>,
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
        /// Optional: External font URLs to download and rasterize
        #[arg(long, value_name = "URL")]
        font: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env())
        .init();

    let args = Cli::parse();
    let config = Config::load(args.config)?;

    match args.command {
        Commands::Serve { port } => {
            let mut config = config;
            if let Some(p) = port {
                config.server.port = p;
            }
            info!("Starting server on port {}", config.server.port);
            server::run(config).await?;
        }
        Commands::Convert {
            type_,
            format,
            input,
            font,
        } => {
            let mut config = config;
            if !font.is_empty() {
                config.mermaid.fonts.extend(font.clone());
                config.graphviz.fonts.extend(font.clone());
                config.excalidraw.fonts.extend(font);
            }
            cli::convert(type_, format, input, config, args.cache_dir).await?;
        }
        Commands::Batch {
            format,
            input,
            type_,
            out_dir,
            font,
        } => {
            let mut config = config;
            if !font.is_empty() {
                config.mermaid.fonts.extend(font.clone());
                config.graphviz.fonts.extend(font.clone());
                config.excalidraw.fonts.extend(font);
            }
            cli::batch(format, input, type_, out_dir, config, args.cache_dir).await?;
        }
    }

    Ok(())
}
