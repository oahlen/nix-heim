use std::{fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

use log::{error, info, warn};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub overwrite: bool,
}

impl Entry {
    pub fn is_installed(&self) -> bool {
        if !self.target.is_symlink() {
            return false;
        }

        match fs::read_link(&self.target) {
            Ok(current) => current == *self.source,
            Err(_) => false,
        }
    }

    pub fn exists(&self) -> bool {
        if let Ok(result) = self.target.try_exists()
            && result
        {
            return true;
        }

        false
    }

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

        if self.target.is_symlink() && !self.exists() {
            self.remove_broken_symlink();
        } else if self.exists() && self.overwrite {
            self.remove_target_file();
        }

        match unix_fs::symlink(&self.source, &self.target) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to create symlink {}: {}", self, err);
            }
        }
    }

    fn remove_target_file(&self) {
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

    fn remove_broken_symlink(&self) {
        info!("Overwriting broken symlink at {}", &self.target.display());

        match fs::remove_file(&self.target) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Failed to cleanup broken symlink {}: {}",
                    self.target.display(),
                    err
                );
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
