use std::{fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

use log::{error, warn};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub overwrite: bool,
}

impl Entry {
    pub fn install(&self) {
        if let Some(parent) = self.target.parent() {
            match fs::create_dir_all(parent) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "Failed to create parent directory {}: {}",
                        parent.display(),
                        err
                    );
                    return;
                }
            }
        }

        if self.target_exists() && self.overwrite {
            warn!("Overwriting existing file {}", self.target.display());

            match fs::remove_file(&self.target) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "Failed to remove existing file {}: {}",
                        self.target.display(),
                        err
                    );
                }
            }
        }

        match unix_fs::symlink(&self.source, &self.target) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to create symlink {}: {}", self, err);
            }
        }
    }

    pub fn uninstall(&self) {
        match fs::remove_file(&self.target) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to remove symlink {}: {}", self, err);
            }
        }
    }

    pub fn target_exists(&self) -> bool {
        self.target.symlink_metadata().is_ok()
    }

    pub fn is_installed(&self) -> bool {
        if !self.target.is_symlink() {
            return false;
        }

        match fs::read_link(&self.target) {
            Ok(current) => current == *self.source,
            Err(_) => false,
        }
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}",
            &self.source.display(),
            &self.target.display()
        )
    }
}
