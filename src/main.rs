pub mod alloc;
#[cfg(target_family = "unix")]
mod daemon;
mod homedir;
mod model;
mod serve;
mod update;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Parser)]
#[clap(author, version, about, arg_required_else_help = true)]
#[command(args_conflicts_with_subcommands = true)]
struct Opt {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run server
    Run(BootArgs),
    /// Start server daemon
    #[cfg(target_family = "unix")]
    Start(BootArgs),
    /// Restart server daemon
    #[cfg(target_family = "unix")]
    Restart(BootArgs),
    /// Stop server daemon
    #[cfg(target_family = "unix")]
    Stop,
    /// Show the server daemon process
    #[cfg(target_family = "unix")]
    Status,
    /// Show the server daemon log
    #[cfg(target_family = "unix")]
    Log,
    /// Update the application
    Update,
}

#[derive(Args, Clone, Debug)]
pub struct BootArgs {
    /// Debug mode
    #[clap(short, long)]
    debug: bool,

    /// Bind address
    #[clap(short, long, default_value = "0.0.0.0:8000")]
    bind: SocketAddr,

    /// TLS certificate file
    #[clap(long)]
    tls_cert: Option<PathBuf>,

    /// TLS private key file
    #[clap(long)]
    tls_key: Option<PathBuf>,

    /// API key
    #[clap(short = 'A', long)]
    api_key: Option<String>,

    /// Multiple image submission limits
    #[clap(short = 'M', long, default_value = "3")]
    multi_image_limit: usize,

    /// Funcaptcha model update check
    #[clap(short = 'U', long)]
    update_check: bool,

    /// Funcaptcha model directory
    #[clap(long)]
    model_dir: Option<PathBuf>,

    /// Number of threads (ONNX Runtime)
    #[clap(long, default_value = "1")]
    num_threads: u16,

    /// Execution provider allocator e.g. device, arena (ONNX Runtime)
    #[clap(long, default_value = "device", value_parser = alloc_parser)]
    allocator: ort::AllocatorType,
}

fn main() -> crate::Result<()> {
    let opt = Opt::parse();

    match opt.commands {
        Commands::Run(args) => daemon::run(args)?,
        #[cfg(target_family = "unix")]
        Commands::Start(args) => daemon::start(args)?,
        #[cfg(target_family = "unix")]
        Commands::Restart(args) => daemon::restart(args)?,
        #[cfg(target_family = "unix")]
        Commands::Stop => daemon::stop()?,
        #[cfg(target_family = "unix")]
        Commands::Status => daemon::status(),
        #[cfg(target_family = "unix")]
        Commands::Log => daemon::log()?,
        Commands::Update => update::update()?,
    };

    Ok(())
}

fn alloc_parser(s: &str) -> anyhow::Result<ort::AllocatorType> {
    match s {
        "device" => Ok(ort::AllocatorType::Device),
        "arena" => Ok(ort::AllocatorType::Arena),
        _ => anyhow::bail!("Invalid allocator type"),
    }
}
