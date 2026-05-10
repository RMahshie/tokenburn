mod cli;
mod config;
mod data;
mod ui;
mod util;

use clap::Parser;
use color_eyre::eyre::Result;

use cli::Cli;
use data::{load_dashboard_data, TimeRange};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let range = TimeRange::try_from(&cli)?;
    let data = load_dashboard_data(&range)?;

    if cli.live {
        ui::live::run(data, range, cli.interval)?;
    } else {
        ui::oneshot::run(data, range)?;
    }

    Ok(())
}
