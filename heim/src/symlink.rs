use log::{debug, warn};
use std::{fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

pub struct Symlink {
    pub source: PathBuf,
    pub target: PathBuf,
    pub overwrite: bool,
}

impl Symlink {
    pub fn new(source: PathBuf, target: PathBuf, overwrite: bool) -> Symlink {
        Symlink {
            source,
            target,
            overwrite,
        }
    }

    pub fn target_exists(&self) -> bool {
        self.target.symlink_metadata().is_ok()
    }

    pub fn install(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.target.parent() {
            fs::create_dir_all(parent)?;
        }

        if self.target_exists() {
            if self.target.is_symlink() {
                debug!("Overwriting existing symlink {}", self.target.display());
            } else if self.overwrite {
                warn!("Overwriting existing file {}", self.target.display());
            } else {
                anyhow::bail!("Unable to install entry {}, another file exists", self)
            }

            fs::remove_file(&self.target)?;
        }

        Ok(unix_fs::symlink(&self.source, &self.target)?)
    }

    pub fn uninstall(&self) -> anyhow::Result<()> {
        Ok(fs::remove_file(&self.target)?)
    }
}

impl Display for Symlink {
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

    use crate::tests::tests::{make_symlink, test_dir, verify_symlink, write_file};

    #[test]
    fn install_and_uninstall_works() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");

        let target_dir = base.join("target");
        let target = target_dir.join("source.txt");

        // Act
        let entry = Symlink::new(source, target.clone(), false);

        // Assert
        assert_eq!(verify_symlink(&entry.target, &entry.source), false);

        // Act
        entry.install().unwrap();

        // Assert
        assert!(target.is_symlink());
        assert_eq!(verify_symlink(&entry.target, &entry.source), true);

        // Act
        entry.uninstall().unwrap();

        // Assert
        assert!(!target.exists());
    }

    #[test]
    fn target_exists_true_for_existing_file() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        // Act
        let entry = Symlink::new(source, target.clone(), false);

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
        let entry = Symlink::new(source, target.clone(), true);
        entry.install().unwrap();

        // Assert
        assert!(target.is_symlink());
        assert_eq!(verify_symlink(&entry.target, &entry.source), true);
        assert_eq!(fs::read_to_string(&target).unwrap(), "src");
    }

    #[test]
    fn install_fails_on_existing_on_no_overwrite() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        // Act
        let entry = Symlink::new(source, target.clone(), false);
        let result = entry.install();

        // Assert
        assert!(result.is_err());

        assert!(!target.is_symlink());
        assert_eq!(verify_symlink(&entry.target, &entry.source), false);
        assert_eq!(fs::read_to_string(&target).unwrap(), "target");
    }

    #[test]
    fn install_replaces_existing_symlink() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");

        let other = write_file(&base, "other.txt", "other");
        let target = make_symlink(&base, "target.txt", &other);

        // Act
        let entry = Symlink::new(source, target.clone(), false);
        entry.install().unwrap();

        // Assert
        assert!(target.is_symlink());
        assert_eq!(verify_symlink(&entry.target, &entry.source), true);
        assert_eq!(fs::read_to_string(&target).unwrap(), "src");
    }
}
