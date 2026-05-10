use chrono::{NaiveDate, TimeZone, Utc};
use clap::{Parser, ValueEnum};
use color_eyre::eyre::{eyre, Result};

use crate::data::TimeRange;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "tokenburn",
    version,
    about = "Track token usage and burn for Claude Code and Codex"
)]
pub struct Cli {
    #[arg(long, help = "Run the interactive live dashboard")]
    pub live: bool,

    #[arg(long, value_enum, default_value_t = RangeArg::SevenDays, help = "Time range to display")]
    pub range: RangeArg,

    #[arg(long, value_name = "YYYY-MM-DD", help = "Custom inclusive start date")]
    pub from: Option<NaiveDate>,

    #[arg(long, value_name = "YYYY-MM-DD", help = "Custom inclusive end date")]
    pub to: Option<NaiveDate>,

    #[arg(long, default_value_t = 5, help = "Live refresh interval in seconds")]
    pub interval: u64,

    #[arg(
        long,
        help = "Set Claude Code cleanupPeriodDays to 3650 days for 10 years of local transcript history"
    )]
    pub fix_claude_retention: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RangeArg {
    #[value(name = "24h")]
    TwentyFourHours,
    #[value(name = "7d")]
    SevenDays,
    #[value(name = "30d")]
    ThirtyDays,
    #[value(name = "lifetime")]
    Lifetime,
}

impl TryFrom<&Cli> for TimeRange {
    type Error = color_eyre::Report;

    fn try_from(cli: &Cli) -> Result<Self> {
        match (cli.from, cli.to) {
            (Some(from), Some(to)) => {
                if to < from {
                    return Err(eyre!("--to must be on or after --from"));
                }
                let start = Utc
                    .from_local_datetime(&from.and_hms_opt(0, 0, 0).expect("valid midnight"))
                    .single()
                    .ok_or_else(|| eyre!("invalid --from date"))?;
                let end = Utc
                    .from_local_datetime(&to.and_hms_opt(23, 59, 59).expect("valid end of day"))
                    .single()
                    .ok_or_else(|| eyre!("invalid --to date"))?;
                Ok(TimeRange::Custom { start, end })
            }
            (None, None) => Ok(match cli.range {
                RangeArg::TwentyFourHours => TimeRange::LastHours(24),
                RangeArg::SevenDays => TimeRange::LastDays(7),
                RangeArg::ThirtyDays => TimeRange::LastDays(30),
                RangeArg::Lifetime => TimeRange::Lifetime,
            }),
            _ => Err(eyre!("--from and --to must be provided together")),
        }
    }
}
