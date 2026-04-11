use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use log::debug;
use serde::Deserialize;

use crate::{
    entry::Entry,
    utils::{home, xdg_state_home},
};

#[derive(Debug, Deserialize, Default)]
pub struct Manifest {
    #[serde(default)]
    pub files: Vec<Entry>,
    pub version: i32,
}

pub struct ManifestDelta<'a> {
    pub remove: Vec<&'a Entry>,
    pub skip: Vec<&'a Entry>,
    pub install: Vec<&'a Entry>,
}

impl Manifest {
    pub fn load_previous() -> anyhow::Result<Manifest> {
        let path = state_path()?;

        Ok(if path.exists() {
            Manifest::load(&path).context("Failed to load previously installed manifest")?
        } else {
            Manifest::default()
        })
    }

    pub fn load(path: &Path) -> anyhow::Result<Manifest> {
        let canonical = path
            .canonicalize()
            .with_context(|| format!("Failed to resolve manifest path: {}", path.display()))?;

        let content = std::fs::read_to_string(&canonical)
            .with_context(|| format!("Failed to read manifest: {}", canonical.display()))?;

        let manifest: Manifest = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {}", canonical.display()))?;

        debug!(
            "Parsed manifest {} with version {}",
            path.display(),
            manifest.version
        );

        let home = home()?;

        for entry in &manifest.files {
            validate(entry, &home)?;
        }

        ensure_no_duplicates(&manifest.files)?;
        Ok(manifest)
    }

    pub fn diff<'a>(previous: &'a Manifest, new: &'a Manifest) -> ManifestDelta<'a> {
        let new_by_target: HashMap<&PathBuf, &Entry> =
            new.files.iter().map(|e| (&e.target, e)).collect();

        let prev_by_target: HashMap<&PathBuf, &Entry> =
            previous.files.iter().map(|e| (&e.target, e)).collect();

        let to_remove = previous
            .files
            .iter()
            .filter(|e| !new_by_target.contains_key(&e.target))
            .collect();

        let mut to_skip = Vec::new();
        let mut to_install = Vec::new();

        for entry in &new.files {
            match prev_by_target.get(&entry.target) {
                Some(prev) if prev.source == entry.source => to_skip.push(entry),
                _ => to_install.push(entry),
            }
        }

        ManifestDelta {
            remove: to_remove,
            skip: to_skip,
            install: to_install,
        }
    }
}

fn validate(entry: &Entry, home: &PathBuf) -> anyhow::Result<()> {
    if !entry.source.is_file() {
        return Err(anyhow!(
            "Source path must be a file: {}",
            entry.source.display()
        ));
    }

    if !entry.target.starts_with(home) {
        return Err(anyhow!(
            "Target path must be contained in user home directory: {}",
            entry.source.display()
        ));
    }

    Ok(())
}

fn ensure_no_duplicates(entries: &Vec<Entry>) -> anyhow::Result<()> {
    let mut seen: BTreeMap<&PathBuf, &PathBuf> = BTreeMap::new();

    for entry in entries.as_slice() {
        if let Some(prev_source) = seen.insert(&entry.target, &entry.source) {
            return Err(anyhow!(
                "Duplicate targets found {} and {} both target {}",
                prev_source.display(),
                entry.source.display(),
                entry.target.display()
            ));
        }
    }

    Ok(())
}

fn state_path() -> anyhow::Result<PathBuf> {
    Ok(xdg_state_home()?.join("heim").join("manifest.json"))
}

pub fn save_manifest_to_state(src: &Path) -> anyhow::Result<()> {
    let dest = state_path()?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create state directory: {}", parent.display()))?;
    }

    fs::copy(src, &dest)
        .with_context(|| format!("Failed to save manifest to: {}", dest.display()))?;

    Ok(())
}

pub fn delete_state_manifest() -> anyhow::Result<()> {
    let path = state_path()?;

    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to delete state manifest: {}", path.display()))?;
    }

    Ok(())
}
