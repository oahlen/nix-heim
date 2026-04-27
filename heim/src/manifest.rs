use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use log::{debug, warn};
use tinyjson::JsonValue;

use crate::{entry::FileEntry, state::State, symlink::Symlink};

const SUPPORTED_VERSION: u32 = 1;

#[derive(Default)]
pub struct Manifest {
    pub files: Vec<FileEntry>,
    pub version: u32,
}

pub struct ManifestDelta {
    pub remove: Vec<Symlink>,
    pub install: Vec<Symlink>,
}

impl Manifest {
    fn from_json(value: &JsonValue, variant: &Option<String>) -> anyhow::Result<Manifest> {
        let obj: &HashMap<String, JsonValue> = value
            .get()
            .ok_or_else(|| anyhow!("Expected manifest to be a JSON object"))?;

        let version = obj
            .get("version")
            .and_then(|v| v.get::<f64>())
            .map(|v| *v as u32)
            .ok_or_else(|| anyhow!("Missing or invalid 'version' field in manifest"))?;

        // Perform version check early since we don't know about future changes to the manifest
        if version > SUPPORTED_VERSION {
            anyhow::bail!(
                "Version in supplied manifest is greater than the supported version: {} > {}",
                version,
                SUPPORTED_VERSION
            )
        }

        let files = match obj.get("files") {
            Some(arr_value) => {
                let arr: &Vec<JsonValue> = arr_value
                    .get()
                    .ok_or_else(|| anyhow!("'files' field must be a JSON array"))?;

                arr.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        FileEntry::from_json(v, variant)
                            .with_context(|| format!("Failed to parse file entry at index {i}"))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
            }
            None => Vec::new(),
        };

        Ok(Manifest { files, version })
    }

    pub fn load(path: &Path, home: &PathBuf, variant: &Option<String>) -> anyhow::Result<Manifest> {
        Manifest::load_internal(path, home, false, variant)
    }

    pub fn load_previous(state: &State) -> anyhow::Result<Manifest> {
        let path = state.previous_manifest()?;
        Ok(if path.exists() {
            match Manifest::load_internal(&path, &state.home, true, &None) {
                Ok(manifest) => manifest,
                Err(error) => {
                    warn!("Failed to load previously installed manifest: {}", error);
                    Manifest::default()
                }
            }
        } else {
            Manifest::default()
        })
    }

    fn load_internal(
        path: &Path,
        home: &PathBuf,
        lenient: bool,
        variant: &Option<String>,
    ) -> anyhow::Result<Manifest> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;

        let json: JsonValue = content
            .parse()
            .map_err(|e| anyhow!("Failed to parse manifest: {}: {e}", path.display()))?;

        let manifest = Manifest::from_json(&json, variant)
            .with_context(|| format!("Failed to parse manifest: {}", path.display()))?;

        debug!(
            "Parsed manifest {} with version {}",
            path.display(),
            manifest.version
        );

        if lenient {
            return Ok(manifest);
        }

        for entry in &manifest.files {
            validate(entry, home)?;
        }

        ensure_no_duplicates(&manifest.files)?;

        Ok(manifest)
    }

    pub fn diff(previous: &Manifest, new: &Manifest) -> ManifestDelta {
        let new_by_target: HashMap<&PathBuf, &FileEntry> =
            new.files.iter().map(|e| (&e.target, e)).collect();

        let to_remove = previous
            .files
            .iter()
            .filter(|e| !new_by_target.contains_key(&e.target))
            .map(|e| e.to_symlink())
            .collect();

        let mut to_install = Vec::new();

        for entry in &new.files {
            to_install.push(entry.to_symlink());
        }

        ManifestDelta {
            remove: to_remove,
            install: to_install,
        }
    }
}

fn validate(entry: &FileEntry, home: &PathBuf) -> anyhow::Result<()> {
    for e in &entry.sources {
        if !e.source.is_file() {
            anyhow::bail!("Source path must be a file: {}", e.source.display());
        }
    }

    if entry
        .target
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        anyhow::bail!(
            "Target path must not use relative path traversal: {}",
            entry.target.display()
        );
    }

    if !entry.target.starts_with(home) {
        anyhow::bail!(
            "Target path must be contained in user home directory: {}",
            entry.target.display()
        );
    }

    Ok(())
}

fn ensure_no_duplicates(entries: &Vec<FileEntry>) -> anyhow::Result<()> {
    let mut seen: HashSet<&PathBuf> = HashSet::new();

    for entry in entries.as_slice() {
        if !seen.insert(&entry.target) {
            anyhow::bail!(
                "Duplicate entries found for target {}",
                entry.target.display()
            );
        }
    }

    Ok(())
}

pub fn copy_manifest(src: &Path, dest: &PathBuf) -> anyhow::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    let content =
        fs::read(src).with_context(|| format!("Failed to read manifest: {}", src.display()))?;

    fs::write(dest, content)
        .with_context(|| format!("Failed to save manifest to: {}", dest.display()))?;

    Ok(())
}

pub fn delete_manifest(path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("Failed to delete manifest: {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        entry::{FileEntry, SourceEntry},
        tests::tests::{test_dir, write_file},
    };

    fn make_entry(source: &str, target: &str) -> FileEntry {
        FileEntry::create(PathBuf::from(source), PathBuf::from(target), false)
    }

    #[test]
    fn from_json_parses_valid_manifest_with_no_files() {
        // Arrange
        let json: JsonValue = r#"{"version": 1, "files": []}"#.parse().unwrap();

        // Act
        let manifest = Manifest::from_json(&json, &None).unwrap();

        // Assert
        assert_eq!(manifest.version, 1);
        assert!(manifest.files.is_empty());
    }

    #[test]
    fn from_json_parses_files() {
        // Arrange
        let json: JsonValue = r#"
{
  "version": 1,
  "files": [
    {
      "sources": [
        {
          "source": "/nix/store/abc/foo",
          "name": "default",
          "default": true
        }
      ],
      "target": "/home/user/.config/foo",
      "overwrite": true
    }
  ]
}
"#
        .parse()
        .unwrap();

        // Act
        let manifest = Manifest::from_json(&json, &None).unwrap();

        // Assert
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(
            *manifest.files[0].sources[0].source,
            PathBuf::from("/nix/store/abc/foo")
        );
        assert_eq!(
            manifest.files[0].target,
            PathBuf::from("/home/user/.config/foo")
        );
        assert!(manifest.files[0].overwrite);
    }

    #[test]
    fn from_json_defaults_overwrite_to_false() {
        // Arrange
        let json: JsonValue = r#"
{
  "version": 1,
  "files": [
    {
      "sources": [
        {
          "source": "/src",
          "name": "default",
          "default": true
        }
      ],
      "target": "/target"
    }
  ]
}
"#
        .parse()
        .unwrap();

        // Act
        let manifest = Manifest::from_json(&json, &None).unwrap();

        // Assert
        assert!(!manifest.files[0].overwrite);
    }

    #[test]
    fn from_json_returns_empty_files_when_files_key_missing() {
        // Arrange
        let json: JsonValue = r#"{"version": 1}"#.parse().unwrap();

        // Act
        let manifest = Manifest::from_json(&json, &None).unwrap();

        // Assert
        assert!(manifest.files.is_empty());
    }

    #[test]
    fn from_json_returns_error_when_version_missing() {
        // Arrange
        let json: JsonValue = r#"{"files": []}"#.parse().unwrap();

        // Act + Assert
        assert!(Manifest::from_json(&json, &None).is_err());
    }

    #[test]
    fn from_json_returns_error_when_version_greater_than_supported() {
        // Arrange
        let json: JsonValue = format!(r#"{{"version": {}}}"#, SUPPORTED_VERSION + 1)
            .parse()
            .unwrap();

        // Act + Assert
        assert!(Manifest::from_json(&json, &None).is_err());
    }

    #[test]
    fn from_json_returns_error_when_not_an_object() {
        // Arrange
        let json: JsonValue = r#"[]"#.parse().unwrap();

        // Act + Assert
        assert!(Manifest::from_json(&json, &None).is_err());
    }

    #[test]
    fn diff_entry_only_in_previous_is_removed() {
        // Arrange
        let previous = Manifest {
            files: vec![make_entry("/src/a", "/target/a")],
            version: 1,
        };
        let new = Manifest {
            files: vec![],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(&previous, &new);

        // Assert
        assert_eq!(delta.remove.len(), 1);
        assert!(delta.install.is_empty());
    }

    #[test]
    fn diff_entry_only_in_new_is_installed() {
        // Arrange
        let previous = Manifest {
            files: vec![],
            version: 1,
        };
        let new = Manifest {
            files: vec![make_entry("/src/a", "/target/a")],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(&previous, &new);

        // Assert
        assert_eq!(delta.install.len(), 1);
        assert!(delta.remove.is_empty());
    }

    #[test]
    fn diff_entry_with_changed_source_is_reinstalled() {
        // Arrange
        let previous = Manifest {
            files: vec![make_entry("/src/old", "/target/a")],
            version: 1,
        };
        let new = Manifest {
            files: vec![make_entry("/src/new", "/target/a")],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(&previous, &new);

        // Assert
        assert_eq!(delta.install.len(), 1);
        assert_eq!(*delta.install[0].source, PathBuf::from("/src/new"));
        assert!(delta.remove.is_empty());
    }

    #[test]
    fn diff_entry_with_different_sources_is_reinstalled() {
        // Arrange
        let previous = Manifest {
            files: vec![make_entry("/src/old", "/target/a")],
            version: 1,
        };
        let new = Manifest {
            files: vec![FileEntry {
                sources: vec![
                    SourceEntry {
                        name: "A".to_string(),
                        source: PathBuf::from("/src/old"),
                        default: true,
                    },
                    SourceEntry {
                        name: "B".to_string(),
                        source: PathBuf::from("/src/b"),
                        default: false,
                    },
                ],
                target: PathBuf::from("/target/a"),
                overwrite: true,
                variant: Some("B".to_string()),
            }],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(&previous, &new);

        // Assert
        assert_eq!(delta.install.len(), 1);
        assert_eq!(*delta.install[0].source, PathBuf::from("/src/b"));
        assert!(delta.remove.is_empty());
    }

    #[test]
    fn validate_succeeds_for_valid_entry() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "content");
        let home = PathBuf::from("/home/user");
        let entry = FileEntry::create(source, home.join("target.txt"), false);

        // Act + Assert
        assert!(validate(&entry, &home).is_ok());
    }

    #[test]
    fn validate_returns_error_when_source_is_not_a_file() {
        // Arrange
        let entry = make_entry("/nonexistent/path", "/home/user/target");
        let home = PathBuf::from("/home/user");

        // Act + Assert
        assert!(validate(&entry, &home).is_err());
    }

    #[test]
    fn validate_returns_error_when_target_has_relative_component() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "content");
        let entry = FileEntry::create(source, PathBuf::from("/home/user/../../etc/target"), false);
        let home = PathBuf::from("/home/user");

        // Act + Assert
        assert!(validate(&entry, &home).is_err());
    }

    #[test]
    fn validate_returns_error_when_target_outside_home() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "content");
        let entry = FileEntry::create(source, PathBuf::from("/etc/target"), false);
        let home = PathBuf::from("/home/user");

        // Act + Assert
        assert!(validate(&entry, &home).is_err());
    }

    #[test]
    fn ensure_no_duplicates_succeeds_for_unique_targets() {
        // Arrange
        let entries = vec![
            make_entry("/src/a", "/target/x"),
            make_entry("/src/b", "/target/y"),
        ];

        // Act + Assert
        assert!(ensure_no_duplicates(&entries).is_ok());
    }

    #[test]
    fn ensure_no_duplicates_returns_error_for_duplicate_targets() {
        // Arrange
        let entries = vec![
            make_entry("/src/a", "/target/x"),
            make_entry("/src/b", "/target/x"),
        ];

        // Act + Assert
        assert!(ensure_no_duplicates(&entries).is_err());
    }
}
