// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::sync::OnceLock;

use crate::gallery_feed::index::{FeedEntry, parse_index};

include!(concat!(env!("OUT_DIR"), "/gallery_snapshot.rs"));

const SNAPSHOT: &str = include_str!("../../gallery/index.json");

fn resolve_snapshot_file(path: &str) -> Option<&[u8]> {
    SNAPSHOT_FILES
        .iter()
        .find(|(key, _)| *key == path)
        .map(|(_, bytes)| *bytes)
}

#[must_use]
pub(crate) fn snapshot_entries() -> &'static [FeedEntry] {
    static ENTRIES: OnceLock<Vec<FeedEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(
            || match parse_index(SNAPSHOT.as_bytes(), &resolve_snapshot_file) {
                Ok(entries) => entries,
                Err(err) => {
                    tracing::warn!("gallery snapshot failed to parse: {err}");
                    Vec::new()
                }
            },
        )
        .as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::modlist_share::preview_modlist_share_code;

    #[test]
    fn snapshot_holds_the_five_entries() {
        let entries = snapshot_entries();
        let ids: Vec<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "eet-plus-fixes",
                "eet-essentials",
                "iwdee-essentials",
                "bgee-vanilla-plus-no-dlc",
                "bgee-vanilla-plus",
            ]
        );
        for entry in entries {
            assert!(
                preview_modlist_share_code(&entry.code).is_ok(),
                "{} must preview",
                entry.id
            );
        }
    }

    #[test]
    fn snapshot_entries_with_a_cover_carry_its_bytes() {
        let entries = snapshot_entries();
        let mut any_cover = false;
        for entry in entries {
            let cover_key = format!("{}/cover.png", entry.id);
            let expects_cover = SNAPSHOT_FILES.iter().any(|(key, _)| *key == cover_key);
            any_cover |= expects_cover;
            assert_eq!(
                entry.cover_png.is_some(),
                expects_cover,
                "{} cover presence mismatch",
                entry.id
            );
        }
        assert!(any_cover, "at least one entry must carry a cover");
    }
}
