use std::{collections::HashMap, fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

use anyhow::anyhow;
use log::warn;
use tinyjson::JsonValue;

pub struct Entry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub overwrite: bool,
}

impl Entry {
    pub fn from_json(value: &JsonValue) -> anyhow::Result<Entry> {
        let obj: &HashMap<String, JsonValue> = value
            .get()
            .ok_or_else(|| anyhow!("Expected file entry to be a JSON object"))?;

        let source = obj
            .get("source")
            .and_then(|v| v.get::<String>())
            .ok_or_else(|| anyhow!("Missing or invalid 'source' field in file entry"))?;

        let target = obj
            .get("target")
            .and_then(|v| v.get::<String>())
            .ok_or_else(|| anyhow!("Missing or invalid 'target' field in file entry"))?;

        let overwrite = obj
            .get("overwrite")
            .and_then(|v| v.get::<bool>())
            .copied()
            .unwrap_or(false);

        Ok(Entry::new(
            PathBuf::from(source),
            PathBuf::from(target),
            overwrite,
        ))
    }

    pub fn new(source: PathBuf, target: PathBuf, overwrite: bool) -> Entry {
        Entry {
            source,
            target,
            overwrite,
        }
    }

    pub fn install(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.target.parent() {
            fs::create_dir_all(parent)?;
        }

        if self.target_exists() && self.overwrite {
            warn!("Overwriting existing file {}", self.target.display());
            fs::remove_file(&self.target)?;
        }

        Ok(unix_fs::symlink(&self.source, &self.target)?)
    }

    pub fn uninstall(&self) -> anyhow::Result<()> {
        Ok(fs::remove_file(&self.target)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{self};

    use crate::tests::tests::{test_dir, write_file};

    #[test]
    fn install_and_uninstall_works() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");

        let target_dir = base.join("target");
        let target = target_dir.join("source.txt");

        // Act
        let entry = Entry::new(source.clone(), target.clone(), false);

        // Assert
        assert!(!entry.is_installed());

        // Act
        entry.install().unwrap();

        // Assert
        assert!(target.is_symlink());
        assert!(entry.is_installed());

        // Act
        entry.uninstall().unwrap();

        // Assert
        assert!(!target.exists());
    }

    #[test]
    fn is_installed_false_for_regular_file() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        // Act
        let entry = Entry::new(source, target, false);

        // Assert
        assert!(!entry.is_installed());
    }

    #[test]
    fn target_exists_true_for_existing_file() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        // Act
        let entry = Entry::new(source, target.clone(), false);

        // Assert
        assert!(entry.target_exists());
    }

    #[test]
    fn install_replaces_existing_on_overwrite() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        // Act
        let entry = Entry::new(source, target.clone(), true);
        entry.install().unwrap();

        // Assert
        assert!(target.is_symlink());
        assert!(entry.is_installed());
        assert_eq!(fs::read_to_string(&target).unwrap(), "src");
    }

    #[test]
    fn install_fails_on_existing_on_no_overwrite() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        // Act
        let entry = Entry::new(source, target.clone(), false);
        let result = entry.install();

        // Assert
        assert!(result.is_err());

        assert!(!target.is_symlink());
        assert!(!entry.is_installed());
        assert_eq!(fs::read_to_string(&target).unwrap(), "target");
    }
}
