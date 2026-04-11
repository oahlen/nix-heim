use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::LevelFilter;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    pub action: ActionType,

    /// Whether to perform a dry run of the specified action.
    /// Does not perform any file system operations.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Prints more detailed information of the performed actions.
    #[clap(short = 'v', long = "verbosity", action = clap::ArgAction::Count, global = true)]
    verbosity: u8,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ActionType {
    /// Activates all managed dotfiles referenced by the supplied manifest.
    Activate {
        /// Path to the manifest json file.
        manifest: PathBuf,
    },
    /// Deactivates all managed dotfiles referenced by the supplied manifest.
    Deacitvate {
        /// Path to the manifest json file.
        manifest: PathBuf,
    },
}

impl Args {
    pub fn parse_args() -> Args {
        let mut cli = Args::parse();
        cli.verbosity = std::cmp::min(3, cli.verbosity);
        cli
    }

    pub fn init_logger(&self) {
        env_logger::builder().filter_level(self.log_level()).init();
    }

    fn log_level(&self) -> LevelFilter {
        if self.dry_run {
            return match self.verbosity {
                3 => LevelFilter::Trace,
                _ => LevelFilter::Debug,
            };
        }

        match self.verbosity {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            3 => LevelFilter::Trace,
            _ => LevelFilter::Warn,
        }
    }
}
