// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use super::cache::{cache_dir, store_cached_index};
use super::index::{FeedEntry, MAX_INDEX_BYTES, parse_index};

pub(crate) const DEFAULT_INDEX_URL: &str = "https://raw.githubusercontent.com/Born2BSalty/Born2BSaltys_Infinity_Orchestrator/main/gallery/index.json";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug)]
pub(crate) enum FetchOutcome {
    Fresh(Vec<FeedEntry>),
    NotModified,
    Failed(String),
}

#[must_use]
pub(crate) fn index_url(override_url: &str) -> String {
    let trimmed = override_url.trim();
    if trimmed.is_empty() {
        DEFAULT_INDEX_URL.to_string()
    } else {
        trimmed.to_string()
    }
}

#[must_use]
pub(crate) fn start_fetch(etag: Option<String>, override_url: &str) -> Receiver<FetchOutcome> {
    let (tx, rx) = mpsc::channel::<FetchOutcome>();
    let cache_dir = cache_dir();
    let url = index_url(override_url);
    let spawn_result = thread::Builder::new()
        .name("gallery-feed-fetch".to_string())
        .spawn(move || {
            let outcome = fetch_once(&url, etag.as_deref(), cache_dir.as_deref());
            let _ = tx.send(outcome);
        });
    if let Err(err) = spawn_result {
        let (tx, rx) = mpsc::channel::<FetchOutcome>();
        let _ = tx.send(FetchOutcome::Failed(format!(
            "gallery index fetch thread failed to start: {err}"
        )));
        return rx;
    }
    rx
}

fn fetch_once(url: &str, etag: Option<&str>, cache_dir: Option<&Path>) -> FetchOutcome {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(CONNECT_TIMEOUT)
        .timeout(TOTAL_TIMEOUT)
        .build();
    let mut request = agent
        .get(url)
        .set("User-Agent", &format!("BIO/{}", env!("CARGO_PKG_VERSION")))
        .set("Accept", "application/json");
    if let Some(etag) = etag {
        request = request.set("If-None-Match", etag);
    }
    match request.call() {
        Ok(response) => {
            let status = response.status();
            let etag = response.header("etag").map(str::to_string);
            let mut body = Vec::new();
            let read_result = response
                .into_reader()
                .take(MAX_INDEX_BYTES as u64 + 1)
                .read_to_end(&mut body);
            if let Err(err) = read_result {
                return FetchOutcome::Failed(format!("gallery index read failed: {err}"));
            }
            interpret_response(status, etag.as_deref(), &body, cache_dir)
        }
        Err(ureq::Error::Status(304, _)) => FetchOutcome::NotModified,
        Err(ureq::Error::Status(code, _)) => {
            FetchOutcome::Failed(format!("gallery index request returned HTTP {code}"))
        }
        Err(ureq::Error::Transport(transport)) => {
            FetchOutcome::Failed(format!("gallery index request failed: {transport}"))
        }
    }
}

pub(crate) fn interpret_response(
    status: u16,
    etag: Option<&str>,
    body: &[u8],
    cache_dir: Option<&Path>,
) -> FetchOutcome {
    if status == 304 {
        return FetchOutcome::NotModified;
    }
    if status != 200 {
        return FetchOutcome::Failed(format!("gallery index request returned HTTP {status}"));
    }
    if body.len() > MAX_INDEX_BYTES {
        return FetchOutcome::Failed("gallery index is larger than 8 MB".to_string());
    }
    match parse_index(body) {
        Ok(entries) => {
            if let Some(dir) = cache_dir {
                if let Some(err) = store_cached_index(dir, body, etag) {
                    tracing::warn!("failed to store gallery index cache: {err}");
                }
            } else {
                tracing::warn!("no config directory available for the gallery cache");
            }
            FetchOutcome::Fresh(entries)
        }
        Err(err) => FetchOutcome::Failed(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::modlist_share::preview_modlist_share_code;
    use crate::gallery_feed::cache::load_cached_index;
    use crate::gallery_feed::index::{IndexEntry, IndexFile, index_label_for_game};
    use crate::platform_defaults;
    use crate::ui::install::gallery::catalog::entries;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempConfigDir {
        root: PathBuf,
    }

    impl TempConfigDir {
        fn new() -> Self {
            let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!(
                "bio_gallery_cache_test_fetch_{}_{counter}",
                std::process::id()
            ));
            std::fs::create_dir_all(&root).expect("create temp config dir");
            platform_defaults::set_config_dir_override(Some(root.clone()));
            Self { root }
        }
    }

    impl Drop for TempConfigDir {
        fn drop(&mut self) {
            platform_defaults::clear_config_dir_override_if(&self.root);
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn valid_index_bytes() -> Vec<u8> {
        let catalog_entry = &entries()[0];
        let code = catalog_entry.code.clone();
        let preview = preview_modlist_share_code(&code).expect("code previews");
        let game = crate::gallery_feed::index::game_from_index_label(&preview.game_install)
            .expect("known game label");
        let index = IndexFile {
            entries: vec![IndexEntry {
                id: "fetch-test-entry".to_string(),
                name: "Fetch Test".to_string(),
                author: "Author".to_string(),
                description: "A description.".to_string(),
                tags: vec!["Tag".to_string()],
                game: index_label_for_game(game).to_string(),
                featured: true,
                version: "1.0.0".to_string(),
                requirements: None,
                code,
                cover_png_base64: None,
            }],
        };
        serde_json::to_vec(&index).expect("serialize index")
    }

    #[test]
    fn a_304_is_not_modified_and_writes_nothing() {
        let guard = TempConfigDir::new();
        let outcome = interpret_response(304, None, &[], cache_dir().as_deref());
        assert!(matches!(outcome, FetchOutcome::NotModified));
        assert!(load_cached_index().is_none());
        drop(guard);
    }

    #[test]
    fn a_200_with_a_valid_index_is_fresh_and_cached() {
        let guard = TempConfigDir::new();
        let body = valid_index_bytes();
        let outcome = interpret_response(200, Some("etag-value"), &body, cache_dir().as_deref());
        match outcome {
            FetchOutcome::Fresh(list) => assert_eq!(list.len(), 1),
            other => panic!("expected Fresh, got {other:?}"),
        }
        let cached = load_cached_index().expect("cache present");
        assert_eq!(cached.bytes, body);
        assert_eq!(cached.etag, Some("etag-value".to_string()));
        drop(guard);
    }

    #[test]
    fn a_200_with_invalid_json_fails_and_leaves_the_cache_untouched() {
        let guard = TempConfigDir::new();
        let outcome = interpret_response(200, None, b"not json", cache_dir().as_deref());
        assert!(matches!(outcome, FetchOutcome::Failed(_)));
        assert!(load_cached_index().is_none());
        drop(guard);
    }

    #[test]
    fn a_500_fails() {
        let outcome = interpret_response(500, None, &[], None);
        assert!(matches!(outcome, FetchOutcome::Failed(_)));
    }

    #[test]
    fn an_oversized_body_fails() {
        let body = vec![b'a'; MAX_INDEX_BYTES + 1];
        let outcome = interpret_response(200, None, &body, None);
        assert!(matches!(outcome, FetchOutcome::Failed(_)));
    }

    #[test]
    fn index_url_prefers_a_non_empty_override() {
        assert_eq!(index_url(""), DEFAULT_INDEX_URL);
        assert_eq!(index_url("   "), DEFAULT_INDEX_URL);
        assert_eq!(
            index_url(" https://example.invalid/gallery/index.json \n"),
            "https://example.invalid/gallery/index.json"
        );
    }
}
