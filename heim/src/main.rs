use log::warn;

use crate::{
    action::Action,
    args::{ActionType, Args},
};

mod action;
mod args;
mod entry;
mod manifest;
mod utils;

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse_args();
    args.init_logger();

    if args.dry_run {
        warn!("Running in dry-run mode");
    }

    match args.action {
        ActionType::Activate { manifest } => Action::new(manifest, args.dry_run).activate(),
        ActionType::Deactivate { manifest } => Action::new(manifest, args.dry_run).deactivate(),
    }
}
