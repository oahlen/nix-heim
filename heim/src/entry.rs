use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{Context, anyhow};
use tinyjson::JsonValue;

use crate::symlink::Symlink;

pub struct SourceEntry {
    pub name: String,
    pub source: PathBuf,
    pub default: bool,
}

impl SourceEntry {
    pub fn from_json(value: &JsonValue) -> anyhow::Result<SourceEntry> {
        let obj: &HashMap<String, JsonValue> = value
            .get()
            .ok_or_else(|| anyhow!("Expected source entry to be a JSON object"))?;

        let name = obj
            .get("name")
            .and_then(|v| v.get::<String>())
            .ok_or_else(|| anyhow!("Missing or invalid 'name' field in source entry"))?;

        let source = obj
            .get("source")
            .and_then(|v| v.get::<String>())
            .ok_or_else(|| anyhow!("Missing or invalid 'source' field in source entry"))?;

        let default = obj
            .get("default")
            .and_then(|v| v.get::<bool>())
            .copied()
            .unwrap_or(false);

        Ok(SourceEntry {
            name: name.to_string(),
            source: PathBuf::from(source),
            default,
        })
    }
}

pub struct FileEntry {
    pub sources: Vec<SourceEntry>,
    pub target: PathBuf,
    pub overwrite: bool,
    pub variant: Option<String>,
}

impl FileEntry {
    pub fn from_json(value: &JsonValue, variant: &Option<String>) -> anyhow::Result<FileEntry> {
        let obj: &HashMap<String, JsonValue> = value
            .get()
            .ok_or_else(|| anyhow!("Expected file entry to be a JSON object"))?;

        let target = obj
            .get("target")
            .and_then(|v| v.get::<String>())
            .ok_or_else(|| anyhow!("Missing or invalid 'target' field in file entry"))?;

        let sources = match obj.get("sources") {
            Some(arr_value) => {
                let arr: &Vec<JsonValue> = arr_value
                    .get()
                    .ok_or_else(|| anyhow!("'sources' field must be a JSON array"))?;

                arr.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        SourceEntry::from_json(v)
                            .with_context(|| format!("Failed to parse source entry at index {i}"))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
            }
            None => Vec::new(),
        };

        if sources.is_empty() {
            anyhow::bail!("Found no sources for entry with target {}", target);
        }

        let overwrite = obj
            .get("overwrite")
            .and_then(|v| v.get::<bool>())
            .copied()
            .unwrap_or(false);

        Ok(FileEntry::new(
            sources,
            PathBuf::from(target),
            overwrite,
            variant.clone(),
        ))
    }

    pub fn new(
        sources: Vec<SourceEntry>,
        target: PathBuf,
        overwrite: bool,
        variant: Option<String>,
    ) -> FileEntry {
        FileEntry {
            sources,
            target,
            overwrite,
            variant,
        }
    }

    pub fn to_symlink(&self) -> (Symlink, bool) {
        let (source, installed) = if let Ok(current) = fs::read_link(&self.target)
            && let Some(installed) = self.matches_any(&current)
        {
            (installed, true)
        } else {
            (self.source(), false)
        };

        (
            Symlink::new(source.clone(), self.target.clone(), self.overwrite),
            installed,
        )
    }

    fn source(&self) -> &PathBuf {
        let pos = self
            .variant
            .as_ref()
            .and_then(|variant| self.sources.iter().position(|f| &f.name == variant))
            .or_else(|| self.sources.iter().position(|f| f.default))
            .unwrap_or(0);

        &self.sources[pos].source
    }

    fn matches_any(&self, current: &PathBuf) -> Option<&PathBuf> {
        if let Some(variant) = &self.variant
            && let Some(index) = self.sources.iter().position(|f| &f.name == variant)
        {
            let source = &self.sources[index].source;
            return (current == source).then_some(source);
        }

        self.sources
            .iter()
            .find(|f| &f.source == current)
            .map(|f| &f.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::tests::{make_symlink, test_dir, write_file};

    #[test]
    fn to_symlink_works() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");

        let target_dir = base.join("target");
        let target = target_dir.join("source.txt");

        let entry = FileEntry {
            sources: vec![SourceEntry {
                source: source.clone(),
                name: "Default".to_string(),
                default: true,
            }],
            target: target.clone(),
            overwrite: false,
            variant: None,
        };

        // Act
        let (symlink, installed) = entry.to_symlink();

        // Assert
        assert_eq!(symlink.source, source);
        assert_eq!(symlink.target, target);
        assert_eq!(symlink.overwrite, entry.overwrite);
        assert_eq!(installed, false);
    }

    #[test]
    fn to_symlink_selects_correct_variant() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let new_source = base.join("source_2");

        let target_dir = base.join("target");
        let target = target_dir.join("source.txt");

        let entry = FileEntry {
            sources: vec![
                SourceEntry {
                    source: source.clone(),
                    name: "dark".to_string(),
                    default: true,
                },
                SourceEntry {
                    source: new_source.clone(),
                    name: "light".to_string(),
                    default: false,
                },
            ],
            target: target.clone(),
            overwrite: false,
            variant: Some("light".to_string()),
        };

        // Act
        let (symlink, installed) = entry.to_symlink();

        // Assert
        assert_eq!(symlink.target, target);
        assert_eq!(symlink.source, new_source);
        assert_eq!(symlink.overwrite, entry.overwrite);
        assert_eq!(installed, false);
    }

    #[test]
    fn to_symlink_selects_default() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");
        let new_source = base.join("source_2");

        let target_dir = base.join("target");
        let target = target_dir.join("source.txt");

        let entry = FileEntry {
            sources: vec![
                SourceEntry {
                    source: source.clone(),
                    name: "dark".to_string(),
                    default: false,
                },
                SourceEntry {
                    source: new_source.clone(),
                    name: "light".to_string(),
                    default: true,
                },
            ],
            target: target.clone(),
            overwrite: false,
            variant: None,
        };

        // Act
        let (symlink, installed) = entry.to_symlink();

        // Assert
        assert_eq!(symlink.target, target);
        assert_eq!(symlink.source, new_source);
        assert_eq!(symlink.overwrite, entry.overwrite);
        assert_eq!(installed, false);
    }

    #[test]
    fn to_symlink_selects_already_installed() {
        // Arrange
        let base = test_dir();
        let source = write_file(&base, "source.txt", "src");

        let target_dir = base.join("target");
        let target = make_symlink(&target_dir, "source.txt", &source);

        // Act
        let entry = FileEntry {
            sources: vec![
                SourceEntry {
                    source: source.clone(),
                    name: "dark".to_string(),
                    default: false,
                },
                SourceEntry {
                    source: base.join("source_2.txt"),
                    name: "light".to_string(),
                    default: true,
                },
            ],
            target: target.clone(),
            overwrite: false,
            variant: Some("unknown".to_string()),
        };

        let (symlink, installed) = entry.to_symlink();

        // Assert
        assert_eq!(symlink.target, target);
        assert_eq!(symlink.source, source);
        assert_eq!(symlink.overwrite, entry.overwrite);
        assert_eq!(installed, true);
    }
}
