use anyhow::anyhow;
use std::path::PathBuf;

const HOME: &str = "HOME";
const XDG_STATE_HOME: &str = "XDG_STATE_HOME";

pub struct State {
    pub home: PathBuf,
    state_home: PathBuf,
}

impl State {
    pub fn create() -> anyhow::Result<State> {
        Ok(State::new(home()?, xdg_state_home()?))
    }

    pub fn new(home: PathBuf, state: PathBuf) -> State {
        State {
            home,
            state_home: state,
        }
    }

    pub fn previous_manifest(&self) -> anyhow::Result<PathBuf> {
        Ok(self.state_home.join("heim").join("manifest.json"))
    }
}

fn home() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(std::env::var(HOME).map_err(|_| {
        anyhow!("Could not determine $HOME directory")
    })?))
}

fn xdg_state_home() -> anyhow::Result<PathBuf> {
    Ok(match std::env::var(XDG_STATE_HOME) {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => home()?.join(".local").join("state"),
    })
}
