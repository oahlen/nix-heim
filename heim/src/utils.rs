use anyhow::anyhow;
use std::path::PathBuf;

const HOME: &str = "HOME";
const XDG_STATE_HOME: &str = "XDG_STATE_HOME";

pub fn xdg_state_home() -> anyhow::Result<PathBuf> {
    Ok(match std::env::var(XDG_STATE_HOME) {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => home()?.join(".local").join("state"),
    })
}

pub fn home() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(std::env::var(HOME).map_err(|_| {
        anyhow!("Could not determine $HOME directory")
    })?))
}
