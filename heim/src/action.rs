use std::{collections::HashSet, path::PathBuf};

use anyhow::{Context, anyhow};
use log::{debug, info};

use crate::{
    entry::Entry,
    manifest::{Manifest, delete_state_manifest, save_manifest_to_state},
};

pub struct Action {
    manifest_path: PathBuf,
    dry_run: bool,
}

impl Action {
    pub fn new(manifest_path: PathBuf, dry_run: bool) -> anyhow::Result<Action> {
        let resolved_path = manifest_path.canonicalize().with_context(|| {
            format!(
                "Failed to resolve manifest path: {}",
                manifest_path.display()
            )
        })?;

        Ok(Action {
            manifest_path: resolved_path,
            dry_run,
        })
    }

    pub fn activate(&self) -> anyhow::Result<()> {
        info!(
            "Installing files from manifest: {}",
            &self.manifest_path.display()
        );

        let previous = Manifest::load_previous()?;
        let manifest = Manifest::load(&self.manifest_path)?;
        let delta = Manifest::diff(&previous, &manifest);

        self.pre_flight_check(&delta.install, &delta.remove)?;

        for entry in &delta.remove {
            info!("Removing entry {}", entry);
            if !self.dry_run {
                entry.uninstall();
            }
        }

        for entry in &delta.skip {
            debug!("Skipping unchanged entry {}", entry);
        }

        for entry in &delta.install {
            info!("Installing entry {}", entry);
            if !self.dry_run {
                entry.install();
            }
        }

        if !self.dry_run {
            save_manifest_to_state(&self.manifest_path)?;
        }

        Ok(())
    }

    fn pre_flight_check(&self, to_install: &[&Entry], to_remove: &[&Entry]) -> anyhow::Result<()> {
        let excluded_targets: HashSet<&PathBuf> = to_remove.iter().map(|s| &s.target).collect();

        let conflicts: Vec<_> = to_install
            .iter()
            .filter(|entry| entry.exists() && !excluded_targets.contains(&entry.target))
            .filter(|entry| !entry.overwrite)
            .collect();

        if !conflicts.is_empty() {
            let listing: Vec<String> = conflicts
                .iter()
                .map(|e| format!("  {}", e.target.display()))
                .collect();
            return Err(anyhow!(
                "Cannot install, the following target files already exist:\n{}",
                listing.join("\n")
            ));
        }

        Ok(())
    }

    pub fn deactivate(&self) -> anyhow::Result<()> {
        info!(
            "Uninstalling files from manifest: {}",
            &self.manifest_path.display()
        );

        let previous = Manifest::load_previous()?;
        let manifest = Manifest::load(&self.manifest_path)?;
        let delta = Manifest::diff(&previous, &manifest);

        // Make sure to also remove all untracked files from the previous manifest
        for entry in &delta.remove {
            uninstall_entry(entry, self.dry_run);
        }

        for entry in manifest.files {
            uninstall_entry(&entry, self.dry_run);
        }

        if !self.dry_run {
            delete_state_manifest().context("Failed to delete state manifest")?;
        }

        Ok(())
    }
}

fn uninstall_entry(entry: &Entry, dry_run: bool) {
    if entry.is_installed() {
        info!("Uninstalling entry {}", entry);
        if !dry_run {
            entry.uninstall();
        }
    }
}
