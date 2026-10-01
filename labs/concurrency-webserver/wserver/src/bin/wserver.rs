use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use wserver::handler::Policy;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'd', default_value = ".")]
    basedir: PathBuf,
    #[arg(short)]
    port: u16,
    #[arg(short, default_value_t = 1)]
    threads: usize,
    #[arg(short, default_value_t = 1)]
    buffer: usize,
    #[arg(short, default_value_t)]
    schedule: Policy,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    wserver::server::run(
        args.basedir.leak(),
        args.port,
        args.threads,
        args.buffer,
        args.schedule,
    )?;
    Ok(())
}
