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
    pub fn deserialize(value: &JsonValue) -> anyhow::Result<SourceEntry> {
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
}

impl FileEntry {
    pub fn new(sources: Vec<SourceEntry>, target: PathBuf, overwrite: bool) -> FileEntry {
        FileEntry {
            sources,
            target,
            overwrite,
        }
    }

    pub fn convert_to_symlink(mut self, variant: &Option<String>) -> (Symlink, bool) {
        let (index, installed) = if let Ok(current) = fs::read_link(&self.target)
            && let Some(idx) = self.matching_index(&current, variant)
        {
            (idx, true)
        } else {
            (self.source_index(variant), false)
        };

        let source = self.sources.swap_remove(index).source;
        (Symlink::new(source, self.target, self.overwrite), installed)
    }

    fn source_index(&self, variant: &Option<String>) -> usize {
        variant
            .as_ref()
            .and_then(|variant| self.sources.iter().position(|f| &f.name == variant))
            .or_else(|| self.sources.iter().position(|f| f.default))
            .unwrap_or(0)
    }

    fn matching_index(&self, current: &PathBuf, variant: &Option<String>) -> Option<usize> {
        if let Some(variant) = variant
            && let Some(index) = self.sources.iter().position(|f| &f.name == variant)
        {
            return (current == &self.sources[index].source).then_some(index);
        }

        self.sources.iter().position(|f| &f.source == current)
    }

    pub fn deserialize(value: &JsonValue) -> anyhow::Result<FileEntry> {
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
                        SourceEntry::deserialize(v)
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

        Ok(FileEntry::new(sources, PathBuf::from(target), overwrite))
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
        };

        // Act
        let (symlink, installed) = entry.convert_to_symlink(&None);

        // Assert
        assert_eq!(symlink.source, source);
        assert_eq!(symlink.target, target);
        assert!(!symlink.overwrite);
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
                    source: source,
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
        };

        // Act
        let (symlink, installed) = entry.convert_to_symlink(&Some("light".to_string()));

        // Assert
        assert_eq!(symlink.target, target);
        assert_eq!(symlink.source, new_source);
        assert!(!symlink.overwrite);
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
                    source: source,
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
        };

        // Act
        let (symlink, installed) = entry.convert_to_symlink(&None);

        // Assert
        assert_eq!(symlink.target, target);
        assert_eq!(symlink.source, new_source);
        assert!(!symlink.overwrite);
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
        };

        let (symlink, installed) = entry.convert_to_symlink(&Some("unknown".to_string()));

        // Assert
        assert_eq!(symlink.target, target);
        assert_eq!(symlink.source, source);
        assert!(!symlink.overwrite);
        assert_eq!(installed, true);
    }
}
