use std::path::PathBuf;

use lexopt::prelude::*;
use log::LevelFilter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Args {
    pub action: ActionType,
    pub dry_run: bool,
    pub verbosity: u8,
}

pub enum ActionType {
    Activate { manifest: PathBuf },
    Deactivate { manifest: PathBuf },
}

impl Args {
    pub fn parse() -> Args {
        match parse_args() {
            Ok(args) => args,
            Err(err) => {
                eprintln!("error: {err}");
                std::process::exit(2);
            }
        }
    }

    pub fn log_level(&self) -> LevelFilter {
        if self.dry_run {
            return match self.verbosity {
                3 => LevelFilter::Trace,
                2 => LevelFilter::Debug,
                _ => LevelFilter::Info,
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

fn parse_args() -> Result<Args, lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut dry_run = false;
    let mut verbosity: u8 = 0;
    let mut command: Option<String> = None;
    let mut manifest: Option<PathBuf> = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) if command.is_none() => command = Some(val.string()?),
            Value(val) if manifest.is_none() => manifest = Some(val.into()),
            Long("dry-run") => dry_run = true,
            Short('v') | Long("verbosity") => verbosity = verbosity.saturating_add(1),
            Short('h') | Long("help") => {
                print_help();
                std::process::exit(0);
            }
            Short('V') | Long("version") => {
                println!("heim {VERSION}");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    verbosity = std::cmp::min(3, verbosity);

    let command = command.ok_or("missing command, expected 'activate' or 'deactivate'")?;

    let action = match command.as_str() {
        "activate" => ActionType::Activate {
            manifest: manifest.ok_or("missing <MANIFEST> path for 'activate'")?,
        },
        "deactivate" => ActionType::Deactivate {
            manifest: manifest.ok_or("missing <MANIFEST> path for 'deactivate'")?,
        },
        other => return Err(format!("unknown command '{other}'"))?,
    };

    Ok(Args {
        action,
        dry_run,
        verbosity,
    })
}

fn print_help() {
    println!("heim {VERSION}");
    println!();
    println!("Usage: heim [OPTIONS] <COMMAND> <MANIFEST>");
    println!();
    println!("Commands:");
    println!("  activate    Activates all managed dotfiles referenced by the supplied manifest");
    println!("  deactivate  Deactivates all managed dotfiles referenced by the supplied manifest");
    println!();
    println!("Arguments:");
    println!("  <MANIFEST>  Path to the manifest JSON file");
    println!();
    println!("Options:");
    println!(
        "      --dry-run       Whether to perform a dry run of the specified action. Does not perform any file system operations"
    );
    println!("  -v, --verbosity...  Prints more detailed information of the performed actions");
    println!("  -h, --help          Print help information");
    println!("  -V, --version       Print version information");
}
