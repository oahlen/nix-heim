use anyhow::anyhow;
use log::{debug, warn};
use std::{collections::HashMap, fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};
use tinyjson::JsonValue;

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

    pub fn is_installed(&self) -> bool {
        if !self.target.is_symlink() {
            return false;
        }

        match fs::read_link(&self.target) {
            Ok(current) => current == *self.source,
            Err(_) => false,
        }
    }

    pub fn install(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.target.parent() {
            fs::create_dir_all(parent)?;
        }

        if let Ok(meta) = self.target.symlink_metadata() {
            if meta.is_symlink() {
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

    pub fn deserialize(value: &JsonValue) -> anyhow::Result<Symlink> {
        let obj: &HashMap<String, JsonValue> = value
            .get()
            .ok_or_else(|| anyhow!("Expected symlink entry to be a JSON object"))?;

        let source = obj
            .get("source")
            .and_then(|v| v.get::<String>())
            .ok_or_else(|| anyhow!("Missing or invalid 'source' field in symlink entry"))?;

        let target = obj
            .get("target")
            .and_then(|v| v.get::<String>())
            .ok_or_else(|| anyhow!("Missing or invalid 'target' field in symlink entry"))?;

        Ok(Symlink::new(
            PathBuf::from(source),
            PathBuf::from(target),
            false,
        ))
    }

    pub fn serialize(&self) -> JsonValue {
        let mut obj = HashMap::new();

        obj.insert(
            "source".to_string(),
            JsonValue::String(self.source.to_string_lossy().into_owned()),
        );

        obj.insert(
            "target".to_string(),
            JsonValue::String(self.target.to_string_lossy().into_owned()),
        );

        JsonValue::Object(obj)
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

    use crate::tests::tests::{make_symlink, test_dir, write_file};

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
        assert_eq!(entry.is_installed(), false);

        // Act
        entry.install().unwrap();

        // Assert
        assert!(target.is_symlink());
        assert_eq!(entry.is_installed(), true);

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
        let entry = Symlink::new(source, target, false);

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
        assert_eq!(entry.is_installed(), true);
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
        assert_eq!(entry.is_installed(), false);
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
        assert_eq!(entry.is_installed(), true);
        assert_eq!(fs::read_to_string(&target).unwrap(), "src");
    }

    #[test]
    fn symlink_serialize_and_deserialize_round_trips() {
        // Arrange
        let symlink = Symlink::new(
            PathBuf::from("/nix/store/abc/foo"),
            PathBuf::from("/home/user/.config/foo"),
            false,
        );

        // Act
        let json = symlink.serialize();
        let restored = Symlink::deserialize(&json).unwrap();

        // Assert
        assert_eq!(restored.source, symlink.source);
        assert_eq!(restored.target, symlink.target);
    }

    #[test]
    fn symlink_deserialize_returns_error_when_source_missing() {
        // Arrange
        let json: JsonValue = r#"{"target": "/home/user/foo"}"#.parse().unwrap();

        // Act + Assert
        assert!(Symlink::deserialize(&json).is_err());
    }

    #[test]
    fn symlink_deserialize_returns_error_when_target_missing() {
        // Arrange
        let json: JsonValue = r#"{"source": "/nix/store/abc/foo"}"#.parse().unwrap();

        // Act + Assert
        assert!(Symlink::deserialize(&json).is_err());
    }
}
