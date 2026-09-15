// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::path::{Path, PathBuf};

use crate::platform_defaults::app_config_dir;
use crate::registry::errors::RegistryError;
use crate::registry::workspace_model::ModlistWorkspaceState;

const MODLISTS_DIR: &str = "modlists";

const WORKSPACE_FILE_NAME: &str = "workspace.json";

#[cfg(test)]
thread_local! {
    static MODLIST_DATA_ROOT: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_modlist_data_root(root: Option<PathBuf>) {
    MODLIST_DATA_ROOT.with(|slot| *slot.borrow_mut() = root);
}

#[cfg(test)]
pub(crate) fn clear_modlist_data_root_if(root: &Path) {
    MODLIST_DATA_ROOT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.as_deref() == Some(root) {
            *slot = None;
        }
    });
}

#[cfg(test)]
fn modlist_data_root() -> Option<PathBuf> {
    MODLIST_DATA_ROOT.with(|slot| slot.borrow().clone())
}

#[cfg(not(test))]
const fn modlist_data_root() -> Option<PathBuf> {
    None
}

#[must_use]
pub fn modlist_data_dir(modlist_id: &str) -> PathBuf {
    modlist_data_root()
        .unwrap_or_else(|| {
            app_config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(MODLISTS_DIR)
        })
        .join(modlist_id)
}

#[derive(Debug, Clone)]
pub struct WorkspaceStore {
    path: PathBuf,
}

impl WorkspaceStore {
    #[must_use]
    pub fn new_for_id(modlist_id: &str) -> Self {
        let path = modlist_data_dir(modlist_id).join(WORKSPACE_FILE_NAME);
        Self { path }
    }

    pub fn new_with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<ModlistWorkspaceState, RegistryError> {
        let raw = match std::fs::read_to_string(&self.path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(RegistryError::corrupt(
                    self.path.clone(),
                    "workspace file is missing".to_string(),
                ));
            }
            Err(err) => return Err(RegistryError::Io(err)),
        };
        match serde_json::from_str::<ModlistWorkspaceState>(&raw) {
            Ok(state) => Ok(state),
            Err(parse_err) => Err(RegistryError::corrupt(
                self.path.clone(),
                parse_err.to_string(),
            )),
        }
    }

    pub fn save(&self, state: &ModlistWorkspaceState) -> Result<(), RegistryError> {
        let raw = serde_json::to_string_pretty(state)?;

        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(RegistryError::Io)?;
        }

        let tmp_path = self.path.with_extension("json.tmp");
        std::fs::write(&tmp_path, raw.as_bytes()).map_err(RegistryError::Io)?;
        std::fs::rename(&tmp_path, &self.path).map_err(RegistryError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static C: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        let n = C.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "bio_workspace_test_{}_{}_{}",
            std::process::id(),
            n,
            label
        ))
    }

    #[test]
    fn save_creates_modlists_subdir() {
        let root = temp_root("subdir");
        let path = root.join("modlists/ABCDEFGHIJKL/workspace.json");
        let store = WorkspaceStore::new_with_path(&path);
        let state = ModlistWorkspaceState {
            last_share_code: Some("X".to_string()),
            ..Default::default()
        };
        store.save(&state).expect("save");
        assert!(path.exists(), "workspace file written");
        assert!(path.parent().unwrap().exists(), "modlists subdir created");
        let loaded = store.load().expect("load");
        assert_eq!(loaded.last_share_code.as_deref(), Some("X"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_workspace_returns_corrupt_error() {
        let root = temp_root("missing");
        let path = root.join("modlists/MISSING/workspace.json");
        let store = WorkspaceStore::new_with_path(&path);
        match store.load() {
            Err(RegistryError::Corrupt { .. }) => {}
            other => panic!("expected Corrupt for missing file, got {other:?}"),
        }
    }

    #[test]
    fn corrupt_workspace_returns_corrupt_error() {
        let root = temp_root("corrupt");
        let path = root.join("modlists/CORRUPT/workspace.json");
        std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        std::fs::write(&path, b"{ not json").expect("write garbage");
        let store = WorkspaceStore::new_with_path(&path);
        match store.load() {
            Err(RegistryError::Corrupt { .. }) => {}
            other => panic!("expected Corrupt, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn data_root_override_is_per_thread() {
        let root = temp_root("per_thread");
        set_modlist_data_root(Some(root.clone()));

        assert!(modlist_data_dir("X").starts_with(&root));

        let other_thread_path = std::thread::spawn(modlist_data_root_default_thread_dir)
            .join()
            .unwrap();
        assert!(!other_thread_path.starts_with(&root));

        set_modlist_data_root(None);
    }

    fn modlist_data_root_default_thread_dir() -> PathBuf {
        modlist_data_dir("X")
    }

    #[test]
    fn clear_only_matches_its_own_root() {
        let root_a = temp_root("clear_a");
        let root_b = temp_root("clear_b");
        set_modlist_data_root(Some(root_a.clone()));

        clear_modlist_data_root_if(&root_b);
        assert!(modlist_data_dir("X").starts_with(&root_a));

        clear_modlist_data_root_if(&root_a);
        assert!(!modlist_data_dir("X").starts_with(&root_a));
    }
}
