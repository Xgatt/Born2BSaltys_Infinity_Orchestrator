// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::fs;
use std::path::{Path, PathBuf};

use crate::platform_defaults;

use super::index::MAX_INDEX_BYTES;

pub(crate) const CACHE_DIR_NAME: &str = "gallery";
pub(crate) const INDEX_FILE_NAME: &str = "index.json";
pub(crate) const ETAG_FILE_NAME: &str = "index.etag";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CachedIndex {
    pub(crate) bytes: Vec<u8>,
    pub(crate) etag: Option<String>,
}

#[must_use]
pub(crate) fn cache_dir() -> Option<PathBuf> {
    Some(platform_defaults::app_config_dir()?.join(CACHE_DIR_NAME))
}

#[must_use]
pub(crate) fn load_cached_index() -> Option<CachedIndex> {
    let dir = cache_dir()?;
    let index_path = dir.join(INDEX_FILE_NAME);
    let metadata = fs::metadata(&index_path).ok()?;
    let Ok(len) = usize::try_from(metadata.len()) else {
        return None;
    };
    if len > MAX_INDEX_BYTES {
        return None;
    }
    let bytes = fs::read(&index_path).ok()?;
    let etag = fs::read_to_string(dir.join(ETAG_FILE_NAME))
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    Some(CachedIndex { bytes, etag })
}

#[must_use]
pub(crate) fn store_cached_index(dir: &Path, bytes: &[u8], etag: Option<&str>) -> Option<String> {
    if let Err(err) = fs::create_dir_all(dir) {
        return Some(format!(
            "create cache directory failed for {}: {err}",
            dir.display()
        ));
    }
    let index_path = dir.join(INDEX_FILE_NAME);
    let tmp_path = dir.join(format!("{INDEX_FILE_NAME}.tmp"));
    if let Err(err) = fs::write(&tmp_path, bytes) {
        return Some(format!(
            "write cache file failed for {}: {err}",
            tmp_path.display()
        ));
    }
    if let Err(err) = fs::rename(&tmp_path, &index_path) {
        return Some(format!(
            "rename cache file failed for {}: {err}",
            index_path.display()
        ));
    }
    let etag_path = dir.join(ETAG_FILE_NAME);
    match etag {
        Some(etag) if !etag.is_empty() => {
            if let Err(err) = fs::write(&etag_path, etag) {
                return Some(format!(
                    "write etag file failed for {}: {err}",
                    etag_path.display()
                ));
            }
        }
        _ => {
            if let Err(err) = fs::remove_file(&etag_path)
                && err.kind() != std::io::ErrorKind::NotFound
            {
                return Some(format!(
                    "remove etag file failed for {}: {err}",
                    etag_path.display()
                ));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempConfigDir {
        root: PathBuf,
    }

    impl TempConfigDir {
        fn new() -> Self {
            let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!(
                "bio_gallery_cache_test_cache_{}_{counter}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("create temp config dir");
            platform_defaults::set_config_dir_override(Some(root.clone()));
            Self { root }
        }
    }

    impl Drop for TempConfigDir {
        fn drop(&mut self) {
            platform_defaults::clear_config_dir_override_if(&self.root);
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn index_path_under(root: &Path) -> PathBuf {
        root.join(CACHE_DIR_NAME).join(INDEX_FILE_NAME)
    }

    #[test]
    fn load_returns_none_when_nothing_is_cached() {
        let guard = TempConfigDir::new();
        assert!(load_cached_index().is_none());
        drop(guard);
    }

    #[test]
    fn store_then_load_round_trips_bytes_and_etag() {
        let guard = TempConfigDir::new();
        let error = store_cached_index(
            &cache_dir().expect("cache dir"),
            b"{\"entries\":[]}",
            Some("abc123"),
        );
        assert_eq!(error, None);
        let loaded = load_cached_index().expect("cache present");
        assert_eq!(loaded.bytes, b"{\"entries\":[]}");
        assert_eq!(loaded.etag, Some("abc123".to_string()));
        drop(guard);
    }

    #[test]
    fn store_without_an_etag_removes_a_stale_etag_file() {
        let guard = TempConfigDir::new();
        let _ = store_cached_index(
            &cache_dir().expect("cache dir"),
            b"{\"entries\":[]}",
            Some("abc123"),
        );
        let error = store_cached_index(&cache_dir().expect("cache dir"), b"{\"entries\":[]}", None);
        assert_eq!(error, None);
        let loaded = load_cached_index().expect("cache present");
        assert_eq!(loaded.etag, None);
        drop(guard);
    }

    #[test]
    fn load_ignores_an_oversized_index_file() {
        let guard = TempConfigDir::new();
        let dir = cache_dir().expect("cache dir");
        fs::create_dir_all(&dir).expect("create cache dir");
        let oversized = vec![b'a'; MAX_INDEX_BYTES + 1];
        fs::write(index_path_under(&guard.root), oversized).expect("write oversized file");
        assert!(load_cached_index().is_none());
        drop(guard);
    }

    #[test]
    fn store_replaces_the_index_atomically() {
        let guard = TempConfigDir::new();
        let _ = store_cached_index(&cache_dir().expect("cache dir"), b"first", None);
        let _ = store_cached_index(&cache_dir().expect("cache dir"), b"second", None);
        let dir = cache_dir().expect("cache dir");
        let tmp_path = dir.join(format!("{INDEX_FILE_NAME}.tmp"));
        assert!(!tmp_path.exists());
        let loaded = load_cached_index().expect("cache present");
        assert_eq!(loaded.bytes, b"second");
        drop(guard);
    }
}
