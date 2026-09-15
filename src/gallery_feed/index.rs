// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::app::modlist_share::{base64_standard_decode, preview_modlist_share_code};
use crate::registry::model::Game;
use crate::ui::install::gallery::catalog::requirements_for;

pub(crate) const MAX_INDEX_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_COVER_BYTES: usize = 150 * 1024;
pub(crate) const MAX_DESCRIPTION_CHARS: usize = 500;
pub(crate) const MAX_NAME_CHARS: usize = 80;
pub(crate) const MAX_AUTHOR_CHARS: usize = 80;
pub(crate) const MAX_VERSION_CHARS: usize = 20;
pub(crate) const MAX_TAG_COUNT: usize = 6;
pub(crate) const MAX_TAG_CHARS: usize = 20;
pub(crate) const MAX_CODE_BYTES: usize = 256 * 1024;
pub(crate) const MIN_COVER_WIDTH: u32 = 460;
pub(crate) const MAX_COVER_WIDTH: u32 = 920;
const COVER_ASPECT_HEIGHT: u32 = 215;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IndexFile {
    pub(crate) entries: Vec<IndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IndexEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) author: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
    pub(crate) game: String,
    pub(crate) featured: bool,
    pub(crate) version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) requirements: Option<String>,
    pub(crate) code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cover_png_base64: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedEntry {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub game: Game,
    pub featured: bool,
    pub version: String,
    pub requirements: String,
    pub code: String,
    pub cover_png: Option<Vec<u8>>,
}

#[must_use]
pub(crate) fn game_from_index_label(label: &str) -> Option<Game> {
    match label {
        "BGEE" => Some(Game::BGEE),
        "BG2EE" => Some(Game::BG2EE),
        "IWDEE" => Some(Game::IWDEE),
        "EET" => Some(Game::EET),
        _ => None,
    }
}

#[must_use]
pub(crate) const fn index_label_for_game(game: Game) -> &'static str {
    game.to_legacy_string()
}

pub(crate) fn is_valid_id(id: &str) -> bool {
    let len = id.chars().count();
    len > 0
        && len <= 64
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

pub(crate) const fn cover_matches_aspect(width: u32, height: u32) -> bool {
    let expected_height = width * COVER_ASPECT_HEIGHT / MIN_COVER_WIDTH;
    let diff = height.abs_diff(expected_height);
    let tolerance = expected_height * 2 / 100;
    diff <= tolerance
}

fn decode_cover(text: &str) -> Option<Vec<u8>> {
    let decoded_bytes = base64_standard_decode(text).ok()?;
    if decoded_bytes.len() > MAX_COVER_BYTES {
        return None;
    }
    let decoded_image = image::load_from_memory(&decoded_bytes).ok()?;
    let width = decoded_image.width();
    let height = decoded_image.height();
    if !(MIN_COVER_WIDTH..=MAX_COVER_WIDTH).contains(&width) {
        return None;
    }
    if !cover_matches_aspect(width, height) {
        return None;
    }
    Some(decoded_bytes)
}

fn char_count_in_range(text: &str, max: usize) -> bool {
    let count = text.chars().count();
    count > 0 && count <= max
}

pub(crate) fn text_field_errors(
    name: &str,
    author: &str,
    version: &str,
    description: &str,
    tags: &[String],
) -> Vec<String> {
    let mut errors = Vec::new();
    if !char_count_in_range(name, MAX_NAME_CHARS) {
        errors.push(format!("name must be 1 to {MAX_NAME_CHARS} characters"));
    }
    if !char_count_in_range(author, MAX_AUTHOR_CHARS) {
        errors.push(format!("author must be 1 to {MAX_AUTHOR_CHARS} characters"));
    }
    if !char_count_in_range(version, MAX_VERSION_CHARS) {
        errors.push(format!(
            "version must be 1 to {MAX_VERSION_CHARS} characters"
        ));
    }
    if !char_count_in_range(description, MAX_DESCRIPTION_CHARS) {
        errors.push(format!(
            "description must be 1 to {MAX_DESCRIPTION_CHARS} characters"
        ));
    }
    if tags.len() > MAX_TAG_COUNT {
        errors.push(format!("at most {MAX_TAG_COUNT} tags are allowed"));
    }
    for tag in tags {
        if !char_count_in_range(tag, MAX_TAG_CHARS) {
            errors.push(format!(
                "tag \"{tag}\" must be 1 to {MAX_TAG_CHARS} characters"
            ));
        }
    }
    errors
}

pub(crate) fn parse_index(bytes: &[u8]) -> Result<Vec<FeedEntry>, String> {
    if bytes.len() > MAX_INDEX_BYTES {
        return Err("gallery index is larger than 8 MB".to_string());
    }
    let index_file: IndexFile = serde_json::from_slice(bytes)
        .map_err(|err| format!("gallery index is not valid JSON: {err}"))?;

    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for entry in index_file.entries {
        if !is_valid_id(&entry.id) || !seen_ids.insert(entry.id.clone()) {
            continue;
        }
        let Some(game) = game_from_index_label(&entry.game) else {
            continue;
        };
        if entry.code.len() > MAX_CODE_BYTES
            || !text_field_errors(
                &entry.name,
                &entry.author,
                &entry.version,
                &entry.description,
                &entry.tags,
            )
            .is_empty()
        {
            continue;
        }
        let Ok(preview) = preview_modlist_share_code(&entry.code) else {
            continue;
        };
        if preview.game_install != index_label_for_game(game) {
            continue;
        }
        let cover_png = match entry.cover_png_base64.as_deref() {
            None => None,
            Some(text) => match decode_cover(text) {
                Some(decoded) => Some(decoded),
                None => continue,
            },
        };
        let requirements = match entry.requirements {
            Some(requirements) if !requirements.is_empty() => requirements,
            _ => requirements_for(game).to_string(),
        };
        result.push(FeedEntry {
            id: entry.id,
            name: entry.name,
            author: entry.author,
            description: entry.description,
            tags: entry.tags,
            game,
            featured: entry.featured,
            version: entry.version,
            requirements,
            code: entry.code,
            cover_png,
        });
    }

    result.sort_by_key(|entry| (!entry.featured, entry.name.to_lowercase()));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::modlist_share::base64_standard_encode;
    use crate::ui::install::gallery::catalog::entries;
    use std::io::Cursor;

    fn entry(id: &str, name: &str, featured: bool, game: &str, code: &str) -> IndexEntry {
        IndexEntry {
            id: id.to_string(),
            name: name.to_string(),
            author: "Author".to_string(),
            description: "A description.".to_string(),
            tags: vec!["Tag".to_string()],
            game: game.to_string(),
            featured,
            version: "1.0.0".to_string(),
            requirements: None,
            code: code.to_string(),
            cover_png_base64: None,
        }
    }

    fn valid_code() -> String {
        entries()[0].code.clone()
    }

    fn cover_base64(width: u32, height: u32) -> String {
        let image = image::RgbaImage::new(width, height);
        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, image::ImageFormat::Png)
            .expect("encode png");
        base64_standard_encode(&buffer.into_inner())
    }

    #[test]
    fn parse_index_accepts_a_valid_two_entry_index() {
        let code = valid_code();
        let index = IndexFile {
            entries: vec![
                entry("eet-essentials-two", "EET Essentials", true, "EET", &code),
                entry("second-list", "Second List", false, "EET", &code),
            ],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_index_drops_an_entry_with_a_bad_id() {
        let code = valid_code();
        let long_id = "a".repeat(65);
        let index = IndexFile {
            entries: vec![
                entry("Uppercase", "One", true, "EET", &code),
                entry(&long_id, "Two", true, "EET", &code),
                entry("", "Three", true, "EET", &code),
            ],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_drops_a_duplicate_id_keeping_the_first() {
        let code = valid_code();
        let index = IndexFile {
            entries: vec![
                entry("dup-id", "First", true, "EET", &code),
                entry("dup-id", "Second", true, "EET", &code),
            ],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "First");
    }

    #[test]
    fn parse_index_drops_an_unknown_game_label() {
        let code = valid_code();
        let index = IndexFile {
            entries: vec![entry("bad-game", "One", true, "SOMEGAME", &code)],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_drops_an_entry_with_an_oversized_code() {
        let oversized = "x".repeat(MAX_CODE_BYTES + 1);
        let index = IndexFile {
            entries: vec![entry("big-code", "One", true, "EET", &oversized)],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_drops_an_entry_with_text_over_the_limits() {
        let code = valid_code();
        let mut long_description = entry("long-description", "One", true, "EET", &code);
        long_description.description = "d".repeat(MAX_DESCRIPTION_CHARS + 1);
        let mut many_tags = entry("many-tags", "Two", true, "EET", &code);
        many_tags.tags = (0..=MAX_TAG_COUNT).map(|n| format!("tag{n}")).collect();
        let mut empty_name = entry("empty-name", "", true, "EET", &code);
        empty_name.name = String::new();
        let index = IndexFile {
            entries: vec![
                long_description,
                many_tags,
                empty_name,
                entry("fine", "Three", true, "EET", &code),
            ],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "fine");
    }

    #[test]
    fn parse_index_drops_an_entry_whose_code_does_not_preview() {
        let index = IndexFile {
            entries: vec![entry("bad-code", "One", true, "EET", "not-a-real-code")],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_drops_an_entry_whose_game_does_not_match_its_code() {
        let code = valid_code();
        let index = IndexFile {
            entries: vec![entry("mismatched-game", "One", true, "BGEE", &code)],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_drops_an_oversized_cover() {
        let code = valid_code();
        let mut fixture = entry("oversized-cover", "One", true, "EET", &code);
        fixture.cover_png_base64 = Some("A".repeat(MAX_COVER_BYTES * 2));
        let index = IndexFile {
            entries: vec![fixture],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_drops_a_cover_with_the_wrong_aspect() {
        let code = valid_code();
        let mut fixture = entry("wrong-aspect", "One", true, "EET", &code);
        fixture.cover_png_base64 = Some(cover_base64(460, 100));
        let index = IndexFile {
            entries: vec![fixture],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_keeps_a_cover_at_460_by_215() {
        let code = valid_code();
        let mut fixture = entry("good-cover", "One", true, "EET", &code);
        fixture.cover_png_base64 = Some(cover_base64(460, 215));
        let index = IndexFile {
            entries: vec![fixture],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert_eq!(result.len(), 1);
        assert!(result[0].cover_png.is_some());
    }

    #[test]
    fn parse_index_drops_a_cover_narrower_than_460() {
        let code = valid_code();
        let mut fixture = entry("narrow-cover", "One", true, "EET", &code);
        fixture.cover_png_base64 = Some(cover_base64(400, 187));
        let index = IndexFile {
            entries: vec![fixture],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_index_errors_on_invalid_json() {
        let result = parse_index(b"not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_index_errors_on_an_oversized_index() {
        let bytes = vec![b'a'; MAX_INDEX_BYTES + 1];
        let result = parse_index(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn parse_index_sorts_featured_first_then_name_case_insensitive() {
        let code = valid_code();
        let index = IndexFile {
            entries: vec![
                entry("z-entry", "zebra", false, "EET", &code),
                entry("a-entry", "Apple", true, "EET", &code),
                entry("m-entry", "Mango", false, "EET", &code),
                entry("b-entry", "banana", true, "EET", &code),
            ],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        let names: Vec<&str> = result.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, vec!["Apple", "banana", "Mango", "zebra"]);
    }

    #[test]
    fn requirements_fall_back_to_the_game_default_when_absent() {
        let code = valid_code();
        let index = IndexFile {
            entries: vec![entry("no-reqs", "One", true, "EET", &code)],
        };
        let bytes = serde_json::to_vec(&index).expect("serialize");
        let result = parse_index(&bytes).expect("parse ok");
        assert_eq!(result[0].requirements, requirements_for(Game::EET));
    }

    #[test]
    fn description_char_cap_matches_the_edit_modlist_dialog_limit() {
        assert_eq!(MAX_DESCRIPTION_CHARS, 500);
    }

    #[test]
    fn game_labels_round_trip() {
        for game in [Game::BGEE, Game::BG2EE, Game::IWDEE, Game::EET] {
            let label = index_label_for_game(game);
            assert_eq!(game_from_index_label(label), Some(game));
        }
        assert_eq!(game_from_index_label("NOPE"), None);
    }
}
