use log::{Level, Metadata, Record, warn};

use crate::{
    action::Action,
    args::{ActionType, Args},
    state::State,
};

mod action;
mod args;
mod entry;
mod manifest;
mod state;
mod tests;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(args.log_level());

    if args.dry_run {
        warn!("Running in dry-run mode");
    }

    match args.action {
        ActionType::Activate { manifest } => {
            Action::new(manifest, args.dry_run, State::create()?)?.activate()
        }
        ActionType::Deactivate { manifest } => {
            Action::new(manifest, args.dry_run, State::create()?)?.deactivate()
        }
    }
}
