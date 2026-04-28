use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use log::{debug, warn};
use tinyjson::JsonValue;

use crate::{entry::FileEntry, symlink::Symlink};

const SUPPORTED_VERSION: u32 = 1;
const SUPPORTED_STATE_VERSION: u32 = 1;

#[derive(Default)]
pub struct Manifest {
    pub files: Vec<FileEntry>,
    pub version: u32,
}

pub struct ManifestDelta {
    pub remove: Vec<Symlink>,
    pub install: Vec<(Symlink, bool)>,
}

pub struct StateManifest;

impl StateManifest {
    pub fn load(path: &Path) -> anyhow::Result<Vec<Symlink>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        match Self::deserialize(path) {
            Ok(symlinks) => Ok(symlinks),
            Err(error) => {
                warn!("Failed to load state manifest: {}", error);
                Ok(Vec::new())
            }
        }
    }

    fn deserialize(path: &Path) -> anyhow::Result<Vec<Symlink>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read state manifest: {}", path.display()))?;

        let json: JsonValue = content
            .parse()
            .map_err(|e| anyhow!("Failed to parse state manifest: {}: {e}", path.display()))?;

        let obj: &HashMap<String, JsonValue> = json
            .get()
            .ok_or_else(|| anyhow!("Expected state manifest to be a JSON object"))?;

        let version = obj
            .get("version")
            .and_then(|v| v.get::<f64>())
            .map(|v| *v as u32)
            .ok_or_else(|| anyhow!("Missing or invalid 'version' field in state manifest"))?;

        if version > SUPPORTED_STATE_VERSION {
            anyhow::bail!(
                "State manifest version is greater than supported: {} > {}",
                version,
                SUPPORTED_STATE_VERSION
            );
        }

        let symlinks = match obj.get("files") {
            Some(arr_value) => {
                let arr: &Vec<JsonValue> = arr_value
                    .get()
                    .ok_or_else(|| anyhow!("'files' field must be a JSON array"))?;

                arr.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        Symlink::from_json(v)
                            .with_context(|| format!("Failed to parse symlink entry at index {i}"))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
            }
            None => Vec::new(),
        };

        Ok(symlinks)
    }

    pub fn save(path: &Path, symlinks: &[&Symlink]) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create state directory: {}", parent.display())
            })?;
        }

        let content = Self::serialize(symlinks)?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write state manifest: {}", path.display()))?;

        Ok(())
    }

    fn serialize(symlinks: &[&Symlink]) -> anyhow::Result<String> {
        let files: Vec<JsonValue> = symlinks.iter().map(|s| s.to_json()).collect();

        let mut obj = HashMap::new();
        obj.insert(
            "version".to_string(),
            JsonValue::Number(SUPPORTED_STATE_VERSION as f64),
        );
        obj.insert("files".to_string(), JsonValue::Array(files));

        let content = JsonValue::Object(obj)
            .stringify()
            .map_err(|e| anyhow!("Failed to serialise state manifest: {e}"))?;

        Ok(content)
    }
}

impl Manifest {
    fn from_json(value: &JsonValue) -> anyhow::Result<Manifest> {
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
                        FileEntry::from_json(v)
                            .with_context(|| format!("Failed to parse file entry at index {i}"))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
            }
            None => Vec::new(),
        };

        Ok(Manifest { files, version })
    }

    pub fn load(path: &Path, home: &PathBuf) -> anyhow::Result<Manifest> {
        let manifest = Manifest::deserialize(path)?;

        for entry in &manifest.files {
            validate(entry, home)?;
        }

        ensure_no_duplicates(&manifest.files)?;

        Ok(manifest)
    }

    fn deserialize(path: &Path) -> anyhow::Result<Manifest> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;

        let json: JsonValue = content
            .parse()
            .map_err(|e| anyhow!("Failed to parse manifest: {}: {e}", path.display()))?;

        let manifest = Manifest::from_json(&json)
            .with_context(|| format!("Failed to parse manifest: {}", path.display()))?;

        debug!(
            "Parsed manifest {} with version {}",
            path.display(),
            manifest.version
        );

        Ok(manifest)
    }

    pub fn diff(previous: Vec<Symlink>, new: Manifest, variant: &Option<String>) -> ManifestDelta {
        let to_remove: Vec<Symlink> = {
            let new_targets: HashSet<&PathBuf> = new.files.iter().map(|e| &e.target).collect();

            previous
                .into_iter()
                .filter(|s| !new_targets.contains(&s.target))
                .collect()
        };

        let to_install = new
            .files
            .into_iter()
            .map(|e| e.convert_to_symlink(variant))
            .collect();

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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        entry::{FileEntry, SourceEntry},
        symlink::Symlink,
        tests::tests::{test_dir, write_file},
    };

    fn make_entry(source: &str, target: &str) -> FileEntry {
        FileEntry::create(PathBuf::from(source), PathBuf::from(target), false)
    }

    fn make_symlink_entry(source: &str, target: &str) -> Symlink {
        Symlink::new(PathBuf::from(source), PathBuf::from(target), false)
    }

    #[test]
    fn from_json_parses_valid_manifest_with_no_files() {
        // Arrange
        let json: JsonValue = r#"{"version": 1, "files": []}"#.parse().unwrap();

        // Act
        let manifest = Manifest::from_json(&json).unwrap();

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
        let manifest = Manifest::from_json(&json).unwrap();

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
        let manifest = Manifest::from_json(&json).unwrap();

        // Assert
        assert!(!manifest.files[0].overwrite);
    }

    #[test]
    fn from_json_returns_empty_files_when_files_key_missing() {
        // Arrange
        let json: JsonValue = r#"{"version": 1}"#.parse().unwrap();

        // Act
        let manifest = Manifest::from_json(&json).unwrap();

        // Assert
        assert!(manifest.files.is_empty());
    }

    #[test]
    fn from_json_returns_error_when_version_missing() {
        // Arrange
        let json: JsonValue = r#"{"files": []}"#.parse().unwrap();

        // Act + Assert
        assert!(Manifest::from_json(&json).is_err());
    }

    #[test]
    fn from_json_returns_error_when_version_greater_than_supported() {
        // Arrange
        let json: JsonValue = format!(r#"{{"version": {}}}"#, SUPPORTED_VERSION + 1)
            .parse()
            .unwrap();

        // Act + Assert
        assert!(Manifest::from_json(&json).is_err());
    }

    #[test]
    fn from_json_returns_error_when_not_an_object() {
        // Arrange
        let json: JsonValue = r#"[]"#.parse().unwrap();

        // Act + Assert
        assert!(Manifest::from_json(&json).is_err());
    }

    #[test]
    fn diff_entry_only_in_previous_is_removed() {
        // Arrange
        let previous = vec![make_symlink_entry("/src/a", "/target/a")];
        let new = Manifest {
            files: vec![],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(previous, new, &None);

        // Assert
        assert_eq!(delta.remove.len(), 1);
        assert!(delta.install.is_empty());
    }

    #[test]
    fn diff_entry_only_in_new_is_installed() {
        // Arrange
        let previous = vec![];
        let new = Manifest {
            files: vec![make_entry("/src/a", "/target/a")],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(previous, new, &None);

        // Assert
        assert_eq!(delta.install.len(), 1);
        assert!(delta.remove.is_empty());
    }

    #[test]
    fn diff_entry_with_changed_source_is_reinstalled() {
        // Arrange
        let previous = vec![make_symlink_entry("/src/old", "/target/a")];
        let new = Manifest {
            files: vec![make_entry("/src/new", "/target/a")],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(previous, new, &None);

        // Assert
        assert_eq!(delta.install.len(), 1);
        assert_eq!(*delta.install[0].0.source, PathBuf::from("/src/new"));
        assert!(delta.remove.is_empty());
    }

    #[test]
    fn diff_entry_with_different_sources_is_reinstalled() {
        // Arrange
        let previous = vec![make_symlink_entry("/src/old", "/target/a")];
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
            }],
            version: 1,
        };

        // Act
        let delta = Manifest::diff(previous, new, &Some("B".to_string()));

        // Assert
        assert_eq!(delta.install.len(), 1);
        assert_eq!(*delta.install[0].0.source, PathBuf::from("/src/b"));
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

    #[test]
    fn state_manifest_save_and_load_round_trips() {
        // Arrange
        let base = test_dir();
        let path = base.join("state.json");
        let symlinks = vec![
            Symlink::new(PathBuf::from("/src/a"), PathBuf::from("/target/a"), false),
            Symlink::new(PathBuf::from("/src/b"), PathBuf::from("/target/b"), false),
        ];
        let refs: Vec<&Symlink> = symlinks.iter().collect();

        // Act
        StateManifest::save(&path, &refs).unwrap();
        let loaded = StateManifest::load(&path).unwrap();

        // Assert
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].source, PathBuf::from("/src/a"));
        assert_eq!(loaded[0].target, PathBuf::from("/target/a"));
        assert_eq!(loaded[1].source, PathBuf::from("/src/b"));
        assert_eq!(loaded[1].target, PathBuf::from("/target/b"));
    }

    #[test]
    fn state_manifest_load_returns_empty_when_file_missing() {
        // Arrange
        let base = test_dir();
        let path = base.join("nonexistent.json");

        // Act
        let loaded = StateManifest::load(&path).unwrap();

        // Assert
        assert!(loaded.is_empty());
    }

    #[test]
    fn state_manifest_load_returns_empty_on_unsupported_version() {
        // Arrange
        let base = test_dir();
        let path = base.join("state.json");
        let content = format!(
            r#"{{"version": {}, "files": []}}"#,
            SUPPORTED_STATE_VERSION + 1
        );
        fs::write(&path, content).unwrap();

        // Act — lenient load returns empty vec and warns rather than propagating the error
        let loaded = StateManifest::load(&path).unwrap();

        // Assert
        assert!(loaded.is_empty());
    }

    #[test]
    fn symlink_to_json_and_from_json_round_trips() {
        // Arrange
        let symlink = Symlink::new(
            PathBuf::from("/nix/store/abc/foo"),
            PathBuf::from("/home/user/.config/foo"),
            false,
        );

        // Act
        let json = symlink.to_json();
        let restored = Symlink::from_json(&json).unwrap();

        // Assert
        assert_eq!(restored.source, symlink.source);
        assert_eq!(restored.target, symlink.target);
    }

    #[test]
    fn symlink_from_json_returns_error_when_source_missing() {
        // Arrange
        let json: JsonValue = r#"{"target": "/home/user/foo"}"#.parse().unwrap();

        // Act + Assert
        assert!(Symlink::from_json(&json).is_err());
    }

    #[test]
    fn symlink_from_json_returns_error_when_target_missing() {
        // Arrange
        let json: JsonValue = r#"{"source": "/nix/store/abc/foo"}"#.parse().unwrap();

        // Act + Assert
        assert!(Symlink::from_json(&json).is_err());
    }
}
