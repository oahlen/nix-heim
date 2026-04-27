use std::{collections::HashSet, path::PathBuf};

use anyhow::Context;
use log::{debug, info, warn};

use crate::{
    manifest::{Manifest, copy_manifest, delete_manifest},
    state::State,
    symlink::Symlink,
};

pub struct Action {
    manifest_path: PathBuf,
    dry_run: bool,
    state: State,
    variant: Option<String>,
}

impl Action {
    pub fn new(
        manifest_path: PathBuf,
        dry_run: bool,
        state: State,
        variant: Option<String>,
    ) -> anyhow::Result<Action> {
        let resolved_path = manifest_path.canonicalize().with_context(|| {
            format!(
                "Failed to resolve manifest path: {}",
                manifest_path.display()
            )
        })?;

        Ok(Action {
            manifest_path: resolved_path,
            dry_run,
            state,
            variant,
        })
    }

    pub fn activate(&self) -> anyhow::Result<()> {
        info!(
            "Installing files from manifest: {}",
            &self.manifest_path.display()
        );

        let previous = Manifest::load_previous(&self.state)?;
        let manifest = Manifest::load(&self.manifest_path, &self.state.home, &self.variant)?;
        let delta = Manifest::diff(previous, manifest);

        self.pre_flight_check(&delta.install, &delta.remove)?;

        for (entry, installed) in delta.remove {
            if installed {
                debug!("Removing entry {}", entry);
                if !self.dry_run {
                    entry.uninstall()?;
                }
            }
        }

        for (entry, installed) in &delta.install {
            if !installed {
                info!("Installing entry {}", entry);

                if !self.dry_run {
                    entry.install()?;
                }
            }
        }

        if !self.dry_run {
            match copy_manifest(&self.manifest_path, &self.state.previous_manifest()?) {
                Ok(_) => {}
                Err(error) => warn!("Unable to store state manifest: {}", error),
            }
        }

        Ok(())
    }

    fn pre_flight_check(
        &self,
        to_install: &[(Symlink, bool)],
        to_remove: &[(Symlink, bool)],
    ) -> anyhow::Result<()> {
        let excluded_targets: HashSet<&PathBuf> = to_remove
            .iter()
            .filter(|(_, installed)| *installed)
            .map(|(entry, _)| &entry.target)
            .collect();

        let conflicts: Vec<_> = to_install
            .iter()
            .filter(|(entry, _)| !excluded_targets.contains(&entry.target))
            .filter(|(entry, installed)| !installed && entry.target_exists())
            .filter(|(entry, _)| !entry.overwrite)
            .collect();

        if !conflicts.is_empty() {
            let listing: Vec<String> = conflicts
                .iter()
                .map(|(entry, _)| format!("  {}", entry.target.display()))
                .collect();
            anyhow::bail!(
                "Cannot perform operation, the following target files already exist and are not tracked by nix-heim:\n{}",
                listing.join("\n")
            );
        }

        Ok(())
    }

    pub fn deactivate(&self) -> anyhow::Result<()> {
        info!(
            "Uninstalling files from manifest: {}",
            &self.manifest_path.display()
        );

        let previous = Manifest::load_previous(&self.state)?;
        let manifest = Manifest::load(&self.manifest_path, &self.state.home, &None)?;
        let delta = Manifest::diff(previous, manifest);

        self.pre_flight_check(&delta.install, &delta.remove)?;

        // Make sure to also remove all untracked files from the previous manifest
        for (entry, installed) in delta.remove {
            if installed {
                info!("Uninstalling previous entry {}", entry);

                if !self.dry_run {
                    entry.uninstall()?;
                }
            }
        }

        for (entry, installed) in delta.install {
            if installed {
                info!("Uninstalling entry {}", entry);

                if !self.dry_run {
                    entry.uninstall()?;
                }
            }
        }

        if !self.dry_run {
            match delete_manifest(&self.state.previous_manifest()?)
                .context("Failed to delete state manifest")
            {
                Ok(_) => {}
                Err(error) => {
                    warn!("Unable to clean up old state manifest: {}", error);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use crate::{
        state::State,
        tests::tests::{test_dir, write_file},
    };

    fn write_manifest(dir: &std::path::Path, entries: &[(PathBuf, PathBuf)]) -> PathBuf {
        let files: Vec<String> = entries
            .iter()
            .map(|(src, tgt)| {
                format!(
                    r#"{{"sources": [{{"source":"{}", "name": "default", "default": true}}], "target": "{}"}}"#,
                    src.display(),
                    tgt.display()
                )
            })
            .collect();
        let content = format!(r#"{{"version": 1, "files": [{}]}}"#, files.join(", "));
        let path = dir.join("manifest.json");
        fs::write(&path, content).unwrap();
        path
    }

    fn empty_action(base: &std::path::Path) -> Action {
        let manifest_path = write_manifest(base, &[]);
        let home = State::new(base.join("home"), base.join("state"));
        Action::new(manifest_path, false, home, None).unwrap()
    }

    #[test]
    fn pre_flight_check_succeeds_with_empty_install_list() {
        // Arrange
        let base = test_dir();
        let action = empty_action(&base);

        // Act + Assert
        assert!(action.pre_flight_check(&[], &[]).is_ok());
    }

    #[test]
    fn pre_flight_check_succeeds_when_overwrite_is_true() {
        // Arrange
        let base = test_dir();
        let action = empty_action(&base);
        let existing_target = write_file(&base, "target.txt", "existing");
        let symlink = Symlink::new(base.join("source.txt"), existing_target, true);

        // Act + Assert
        assert!(action.pre_flight_check(&[(symlink, false)], &[]).is_ok());
    }

    #[test]
    fn pre_flight_check_succeeds_when_conflicting_target_is_being_removed() {
        // Arrange
        let base = test_dir();
        let action = empty_action(&base);
        let existing_target = write_file(&base, "target.txt", "existing");

        let install = (
            Symlink::new(base.join("source.txt"), existing_target.clone(), false),
            false,
        );

        let remove = (
            Symlink::new(base.join("source.txt"), existing_target.clone(), false),
            true,
        );

        // Act + Assert
        assert!(action.pre_flight_check(&[install], &[remove]).is_ok());
    }

    #[test]
    fn pre_flight_check_returns_error_when_target_exists() {
        // Arrange
        let base = test_dir();
        let action = empty_action(&base);
        let existing_target = write_file(&base, "target.txt", "existing");
        let symlink = Symlink::new(base.join("source.txt"), existing_target, false);

        // Act + Assert
        assert!(action.pre_flight_check(&[(symlink, false)], &[]).is_err());
    }

    #[test]
    fn activate_installs_files_from_manifest() {
        // Arrange
        let base = test_dir();
        let home = base.join("home");
        let state = base.join("state");
        fs::create_dir_all(&home).unwrap();

        let source = write_file(&base, "source.txt", "content");
        let target = home.join("target.txt");
        let manifest_path = write_manifest(&base, &[(source, target.clone())]);
        let new_manifest_path = state.join("heim").join("manifest.json");

        let action =
            Action::new(manifest_path, false, State::new(home, state.clone()), None).unwrap();

        // Act
        let result = action.activate();

        // Assert
        assert!(result.is_ok());
        assert!(target.is_symlink());
        assert!(new_manifest_path.exists());
    }

    #[test]
    fn activate_dry_run_does_not_install_files() {
        // Arrange
        let base = test_dir();
        let home = base.join("home");
        fs::create_dir_all(&home).unwrap();

        let source = write_file(&base, "source.txt", "content");
        let target = home.join("target.txt");
        let manifest_path = write_manifest(&base, &[(source, target.clone())]);

        let action = Action::new(
            manifest_path,
            true,
            State::new(home, base.join("state")),
            None,
        )
        .unwrap();

        // Act
        let result = action.activate();

        // Assert
        assert!(result.is_ok());
        assert!(!target.exists());
    }

    #[test]
    fn activate_returns_error_on_conflicting_target() {
        // Arrange
        let base = test_dir();
        let home = base.join("home");
        fs::create_dir_all(&home).unwrap();

        let source = write_file(&base, "source.txt", "content");
        let target = write_file(&home, "target.txt", "existing");
        let manifest_path = write_manifest(&base, &[(source, target)]);

        let action = Action::new(
            manifest_path,
            false,
            State::new(home, base.join("state")),
            None,
        )
        .unwrap();

        // Act + Assert
        assert!(action.activate().is_err());
    }

    #[test]
    fn deactivate_removes_installed_symlinks() {
        // Arrange
        let base = test_dir();
        let home = base.join("home");
        fs::create_dir_all(&home).unwrap();

        let source = write_file(&base, "source.txt", "content");
        let target = home.join("target.txt");
        let manifest_path = write_manifest(&base, &[(source, target.clone())]);

        let state = base.join("state");
        let heim_state = state.join("heim");
        fs::create_dir_all(&heim_state).unwrap();
        let old_manifest_path = write_manifest(&heim_state, &[]);

        let action = Action::new(manifest_path, false, State::new(home, state), None).unwrap();
        action.activate().unwrap();
        assert!(target.is_symlink());

        // Act
        let result = action.deactivate();

        // Assert
        assert!(result.is_ok());
        assert!(!target.exists());
        assert!(!old_manifest_path.exists());
    }

    #[test]
    fn deactivate_dry_run_does_not_remove_files() {
        // Arrange
        let base = test_dir();
        let home = base.join("home");
        fs::create_dir_all(&home).unwrap();

        let source = write_file(&base, "source.txt", "content");
        let target = home.join("target.txt");
        let manifest_path = write_manifest(&base, &[(source, target.clone())]);
        let state = base.join("state");

        let action = Action::new(
            manifest_path.clone(),
            false,
            State::new(home.clone(), state.clone()),
            None,
        )
        .unwrap();
        action.activate().unwrap();
        assert!(target.is_symlink());

        // Act
        let dry_action = Action::new(manifest_path, true, State::new(home, state), None).unwrap();
        let result = dry_action.deactivate();

        // Assert
        assert!(result.is_ok());
        assert!(target.is_symlink());
    }
}
