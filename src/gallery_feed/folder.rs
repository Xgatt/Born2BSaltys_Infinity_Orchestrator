// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::ZipArchive;

use crate::app::modlist_biolist;
use crate::app::modlist_share::{base64_standard_encode, preview_modlist_share_code};
use crate::gallery_feed::index::{
    IndexEntry, IndexFile, MAX_CODE_BYTES, MAX_COVER_BYTES, MAX_COVER_WIDTH, MIN_COVER_WIDTH,
    cover_matches_aspect, game_from_index_label, index_label_for_game, is_valid_id,
    text_field_errors,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EntryMeta {
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
}

#[derive(Debug, Default)]
pub struct FolderReport {
    pub errors: Vec<String>,
    pub index_text: Option<String>,
}

fn subfolders(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(read_dir) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut result: Vec<(String, PathBuf)> = read_dir
        .filter_map(Result::ok)
        .filter(|dir_entry| dir_entry.path().is_dir())
        .filter_map(|dir_entry| {
            let name = dir_entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                None
            } else {
                Some((name, dir_entry.path()))
            }
        })
        .collect();
    result.sort_by(|left, right| left.0.cmp(&right.0));
    result
}

fn zip_entries(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut archive =
        ZipArchive::new(Cursor::new(bytes.to_vec())).map_err(|err| err.to_string())?;
    let mut map = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|err| err.to_string())?;
        let name = entry.name().to_string();
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .map_err(|err| err.to_string())?;
        map.insert(name, data);
    }
    Ok(map)
}

fn compare_biolist(disk_bytes: &[u8], code: &str) -> Result<(), String> {
    let generated_bytes = modlist_biolist::build_biolist(code)?;
    let disk_entries = zip_entries(disk_bytes)?;
    let generated_entries = zip_entries(&generated_bytes)?;
    let disk_names: Vec<&String> = disk_entries.keys().collect();
    let generated_names: Vec<&String> = generated_entries.keys().collect();
    if disk_names != generated_names {
        let first_diff = disk_entries
            .keys()
            .find(|name| !generated_entries.contains_key(name.as_str()))
            .or_else(|| {
                generated_entries
                    .keys()
                    .find(|name| !disk_entries.contains_key(name.as_str()))
            })
            .cloned()
            .unwrap_or_default();
        return Err(format!(
            "modlist.biolist entry list differs at {first_diff}"
        ));
    }
    for (name, disk_data) in &disk_entries {
        let generated_data = generated_entries
            .get(name)
            .ok_or_else(|| format!("modlist.biolist entry list differs at {name}"))?;
        if disk_data != generated_data {
            return Err(format!(
                "modlist.biolist entry {name} does not match its code"
            ));
        }
    }
    Ok(())
}

fn validate_cover(cover_path: &Path) -> Result<Option<Vec<u8>>, String> {
    if !cover_path.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(cover_path).map_err(|err| err.to_string())?;
    if bytes.len() > MAX_COVER_BYTES {
        return Err("cover.png is over 150 KB".to_string());
    }
    let decoded = image::load_from_memory(&bytes)
        .map_err(|err| format!("cover.png is not a decodable PNG: {err}"))?;
    let width = decoded.width();
    let height = decoded.height();
    if !(MIN_COVER_WIDTH..=MAX_COVER_WIDTH).contains(&width) {
        return Err(format!(
            "cover.png width {width} is outside {MIN_COVER_WIDTH}..={MAX_COVER_WIDTH}"
        ));
    }
    if !cover_matches_aspect(width, height) {
        return Err(format!(
            "cover.png is {width}x{height}, which is not within 2% of the 460:215 aspect"
        ));
    }
    Ok(Some(bytes))
}

fn validate_meta_fields(id: &str, meta: &EntryMeta, seen_ids: &mut HashSet<String>) -> Vec<String> {
    let mut errors = Vec::new();
    if meta.id != id {
        errors.push(format!(
            "{id}: id \"{}\" does not match the folder name",
            meta.id
        ));
    }
    if !is_valid_id(&meta.id) {
        errors.push(format!(
            "{id}: id \"{}\" is not lowercase letters, digits and dashes",
            meta.id
        ));
    } else if !seen_ids.insert(meta.id.clone()) {
        errors.push(format!("{id}: id \"{}\" is a duplicate", meta.id));
    }
    for error in text_field_errors(
        &meta.name,
        &meta.author,
        &meta.version,
        &meta.description,
        &meta.tags,
    ) {
        errors.push(format!("{id}: {error}"));
    }
    if game_from_index_label(&meta.game).is_none() {
        errors.push(format!(
            "{id}: game \"{}\" is not one of BGEE, BG2EE, IWDEE, EET",
            meta.game
        ));
    }
    errors
}

fn build_entry_for_folder(
    id: &str,
    folder_path: &Path,
    meta: &EntryMeta,
    errors: &mut Vec<String>,
) -> Option<IndexEntry> {
    let game = game_from_index_label(&meta.game)?;

    let biolist_path = folder_path.join("modlist.biolist");
    let code = match modlist_biolist::read_share_code(&biolist_path) {
        Ok(code) if code.len() > MAX_CODE_BYTES => {
            errors.push(format!(
                "{id}: share code is {} bytes, over the {MAX_CODE_BYTES} byte limit",
                code.len()
            ));
            None
        }
        Ok(code) => Some(code),
        Err(err) => {
            errors.push(format!("{id}: {err}"));
            None
        }
    };

    let mut cover_png_base64 = None;
    match validate_cover(&folder_path.join("cover.png")) {
        Ok(Some(bytes)) => cover_png_base64 = Some(base64_standard_encode(&bytes)),
        Ok(None) => {}
        Err(err) => errors.push(format!("{id}: {err}")),
    }

    let code = code?;

    match preview_modlist_share_code(&code) {
        Ok(preview) => {
            if preview.game_install != index_label_for_game(game) {
                errors.push(format!(
                    "{id}: entry.json game \"{}\" does not match the code's game \"{}\"",
                    meta.game, preview.game_install
                ));
            }
            if !preview.unresolved_mods.is_empty() {
                errors.push(format!(
                    "{id}: {} mods have no download source",
                    preview.unresolved_mods.len()
                ));
            }
        }
        Err(err) => errors.push(format!("{id}: {err}")),
    }

    match std::fs::read(&biolist_path) {
        Ok(disk_bytes) => {
            if let Err(err) = compare_biolist(&disk_bytes, &code) {
                errors.push(format!("{id}: {err}"));
            }
        }
        Err(err) => errors.push(format!("{id}: {err}")),
    }

    Some(IndexEntry {
        id: meta.id.clone(),
        name: meta.name.clone(),
        author: meta.author.clone(),
        description: meta.description.clone(),
        tags: meta.tags.clone(),
        game: meta.game.clone(),
        featured: meta.featured,
        version: meta.version.clone(),
        requirements: meta.requirements.clone(),
        code,
        cover_png_base64,
    })
}

#[must_use]
pub fn build_index(root: &Path) -> FolderReport {
    let mut errors = Vec::new();
    let mut entries = Vec::new();
    let mut seen_ids = HashSet::new();

    for (id, folder_path) in subfolders(root) {
        let entry_path = folder_path.join("entry.json");
        let text = match std::fs::read_to_string(&entry_path) {
            Ok(text) => text,
            Err(err) => {
                errors.push(format!("{id}: entry.json missing or unreadable: {err}"));
                continue;
            }
        };
        let meta: EntryMeta = match serde_json::from_str(&text) {
            Ok(meta) => meta,
            Err(err) => {
                errors.push(format!("{id}: entry.json fails to parse: {err}"));
                continue;
            }
        };

        let errors_before = errors.len();
        errors.extend(validate_meta_fields(&id, &meta, &mut seen_ids));
        let candidate = build_entry_for_folder(&id, &folder_path, &meta, &mut errors);

        if errors.len() == errors_before
            && let Some(entry) = candidate
        {
            entries.push(entry);
        }
    }

    if errors.is_empty() {
        let index = IndexFile { entries };
        FolderReport {
            errors,
            index_text: Some(render_index(&index)),
        }
    } else {
        FolderReport {
            errors,
            index_text: None,
        }
    }
}

pub(crate) fn render_index(index: &IndexFile) -> String {
    let mut entries = index.entries.clone();
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    let sorted = IndexFile { entries };
    let mut text = serde_json::to_string_pretty(&sorted).unwrap_or_default();
    text.push('\n');
    text
}

pub fn check_index(root: &Path) -> Result<usize, Vec<String>> {
    let report = build_index(root);
    if !report.errors.is_empty() {
        return Err(report.errors);
    }
    let Some(index_text) = report.index_text else {
        return Err(vec![
            "index.json is stale: run gallery-index build".to_string(),
        ]);
    };
    let committed = std::fs::read_to_string(root.join("index.json"))
        .ok()
        .map(|text| text.replace("\r\n", "\n"));
    if committed.as_deref() != Some(index_text.as_str()) {
        return Err(vec![
            "index.json is stale: run gallery-index build".to_string(),
        ]);
    }
    let parsed: IndexFile = serde_json::from_str(&index_text).unwrap_or(IndexFile {
        entries: Vec::new(),
    });
    Ok(parsed.entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::install::gallery::catalog;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new() -> Self {
            let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "bio_gallery_folder_test_{}_{counter}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("create temp root");
            Self(path)
        }

        fn entry_dir(&self, id: &str) -> PathBuf {
            let dir = self.0.join(id);
            std::fs::create_dir_all(&dir).expect("create entry dir");
            dir
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn write_valid_entry(root: &TempRoot, id: &str) -> String {
        let entry = &catalog::entries()[0];
        let code = entry.code.clone();
        write_entry_with(root, id, id, "EET", &code)
    }

    fn write_entry_with(root: &TempRoot, folder: &str, id: &str, game: &str, code: &str) -> String {
        let dir = root.entry_dir(folder);
        let entry_json = format!(
            "{{\"id\":\"{id}\",\"name\":\"Name\",\"author\":\"Author\",\"description\":\"A description.\",\"tags\":[\"Tag\"],\"game\":\"{game}\",\"featured\":true,\"version\":\"1.0.0\"}}"
        );
        std::fs::write(dir.join("entry.json"), entry_json).expect("write entry.json");
        let bytes = modlist_biolist::build_biolist(code).expect("build biolist");
        std::fs::write(dir.join("modlist.biolist"), bytes).expect("write modlist.biolist");
        code.to_string()
    }

    fn bgee_code() -> String {
        let entry = catalog::entries()
            .iter()
            .find(|entry| entry.id == "bgee-vanilla-plus-no-dlc")
            .expect("bgee entry exists");
        entry.code.clone()
    }

    fn cover_bytes(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbaImage::new(width, height);
        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, image::ImageFormat::Png)
            .expect("encode png");
        buffer.into_inner()
    }

    #[test]
    fn a_valid_folder_builds_one_entry() {
        let root = TempRoot::new();
        write_valid_entry(&root, "eet-essentials");
        let report = build_index(&root.0);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.index_text.is_some());
    }

    #[test]
    fn id_must_match_the_folder_name() {
        let root = TempRoot::new();
        let code = catalog::entries()[0].code.clone();
        write_entry_with(&root, "folder-name", "different-id", "EET", &code);
        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("does not match the folder name"))
        );
    }

    #[test]
    fn an_unknown_game_is_an_error() {
        let root = TempRoot::new();
        let code = catalog::entries()[0].code.clone();
        write_entry_with(&root, "bad-game", "bad-game", "SOMEGAME", &code);
        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("is not one of BGEE"))
        );
    }

    #[test]
    fn a_game_that_differs_from_the_code_is_an_error() {
        let root = TempRoot::new();
        let code = bgee_code();
        write_entry_with(&root, "wrong-game", "wrong-game", "EET", &code);
        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("does not match the code's game"))
        );
    }

    #[test]
    fn a_description_over_500_chars_is_an_error() {
        let root = TempRoot::new();
        let dir = root.entry_dir("too-long");
        let long_description = "a".repeat(501);
        let code = catalog::entries()[0].code.clone();
        let entry_json = format!(
            "{{\"id\":\"too-long\",\"name\":\"Name\",\"author\":\"Author\",\"description\":\"{long_description}\",\"tags\":[],\"game\":\"EET\",\"featured\":true,\"version\":\"1.0.0\"}}"
        );
        std::fs::write(dir.join("entry.json"), entry_json).expect("write entry.json");
        let bytes = modlist_biolist::build_biolist(&code).expect("build biolist");
        std::fs::write(dir.join("modlist.biolist"), bytes).expect("write modlist.biolist");
        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("description must be"))
        );
    }

    #[test]
    fn a_seventh_tag_is_an_error() {
        let root = TempRoot::new();
        let dir = root.entry_dir("many-tags");
        let code = catalog::entries()[0].code.clone();
        let entry_json = "{\"id\":\"many-tags\",\"name\":\"Name\",\"author\":\"Author\",\"description\":\"A description.\",\"tags\":[\"1\",\"2\",\"3\",\"4\",\"5\",\"6\",\"7\"],\"game\":\"EET\",\"featured\":true,\"version\":\"1.0.0\"}".to_string();
        std::fs::write(dir.join("entry.json"), entry_json).expect("write entry.json");
        let bytes = modlist_biolist::build_biolist(&code).expect("build biolist");
        std::fs::write(dir.join("modlist.biolist"), bytes).expect("write modlist.biolist");
        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("at most 6 tags"))
        );
    }

    #[test]
    fn an_unknown_meta_key_is_an_error() {
        let root = TempRoot::new();
        let dir = root.entry_dir("bad-key");
        let entry_json = "{\"id\":\"bad-key\",\"name\":\"Name\",\"author\":\"Author\",\"description\":\"A description.\",\"tags\":[],\"game\":\"EET\",\"featured\":true,\"version\":\"1.0.0\",\"min_bio_version\":\"1\"}";
        std::fs::write(dir.join("entry.json"), entry_json).expect("write entry.json");
        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("fails to parse"))
        );
    }

    #[test]
    fn a_missing_biolist_is_an_error() {
        let root = TempRoot::new();
        let dir = root.entry_dir("no-biolist");
        let entry_json = "{\"id\":\"no-biolist\",\"name\":\"Name\",\"author\":\"Author\",\"description\":\"A description.\",\"tags\":[],\"game\":\"EET\",\"featured\":true,\"version\":\"1.0.0\"}";
        std::fs::write(dir.join("entry.json"), entry_json).expect("write entry.json");
        let report = build_index(&root.0);
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn a_tampered_reference_file_is_an_error() {
        let root = TempRoot::new();
        let code = write_valid_entry(&root, "tampered");
        let dir = root.0.join("tampered");
        let biolist_path = dir.join("modlist.biolist");
        let bytes = std::fs::read(&biolist_path).expect("read biolist");

        let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("open zip");
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            let name = entry.name().to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).expect("read entry");
            if name == "reference/README.txt" {
                data = b"tampered".to_vec();
            }
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file(&name, options).expect("start file");
            std::io::Write::write_all(&mut writer, &data).expect("write file");
        }
        let tampered_bytes = writer.finish().expect("finish").into_inner();
        std::fs::write(&biolist_path, tampered_bytes).expect("write tampered biolist");
        let _ = code;

        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("does not match its code"))
        );
    }

    #[test]
    fn an_oversized_cover_is_an_error() {
        let root = TempRoot::new();
        write_valid_entry(&root, "oversized-cover");
        let dir = root.0.join("oversized-cover");
        std::fs::write(dir.join("cover.png"), vec![0u8; 200 * 1024]).expect("write cover");
        let report = build_index(&root.0);
        assert!(report.errors.iter().any(|err| err.contains("over 150 KB")));
    }

    #[test]
    fn a_wrong_aspect_cover_is_an_error() {
        let root = TempRoot::new();
        write_valid_entry(&root, "wrong-aspect-cover");
        let dir = root.0.join("wrong-aspect-cover");
        std::fs::write(dir.join("cover.png"), cover_bytes(460, 100)).expect("write cover");
        let report = build_index(&root.0);
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("not within 2%"))
        );
    }

    #[test]
    fn a_good_cover_is_embedded_as_base64() {
        let root = TempRoot::new();
        write_valid_entry(&root, "good-cover");
        let dir = root.0.join("good-cover");
        std::fs::write(dir.join("cover.png"), cover_bytes(460, 215)).expect("write cover");
        let report = build_index(&root.0);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let index_text = report.index_text.expect("index text present");
        assert!(index_text.contains("cover_png_base64"));
    }

    #[test]
    fn check_reports_a_missing_index_as_stale() {
        let root = TempRoot::new();
        write_valid_entry(&root, "eet-essentials");
        let result = check_index(&root.0);
        assert_eq!(
            result,
            Err(vec![
                "index.json is stale: run gallery-index build".to_string()
            ])
        );
    }

    #[test]
    fn check_reports_a_differing_index_as_stale() {
        let root = TempRoot::new();
        write_valid_entry(&root, "eet-essentials");
        std::fs::write(root.0.join("index.json"), "{}").expect("write stale index");
        let result = check_index(&root.0);
        assert_eq!(
            result,
            Err(vec![
                "index.json is stale: run gallery-index build".to_string()
            ])
        );
    }

    #[test]
    fn check_passes_a_fresh_index() {
        let root = TempRoot::new();
        write_valid_entry(&root, "eet-essentials");
        let report = build_index(&root.0);
        let index_text = report.index_text.expect("index text present");
        std::fs::write(root.0.join("index.json"), &index_text).expect("write index");
        let result = check_index(&root.0);
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn render_index_sorts_by_id_and_ends_with_a_newline() {
        let index = IndexFile {
            entries: vec![
                IndexEntry {
                    id: "zulu".to_string(),
                    name: "Zulu".to_string(),
                    author: "Author".to_string(),
                    description: "A description.".to_string(),
                    tags: vec![],
                    game: "EET".to_string(),
                    featured: true,
                    version: "1.0.0".to_string(),
                    requirements: None,
                    code: "code".to_string(),
                    cover_png_base64: None,
                },
                IndexEntry {
                    id: "alpha".to_string(),
                    name: "Alpha".to_string(),
                    author: "Author".to_string(),
                    description: "A description.".to_string(),
                    tags: vec![],
                    game: "EET".to_string(),
                    featured: true,
                    version: "1.0.0".to_string(),
                    requirements: None,
                    code: "code".to_string(),
                    cover_png_base64: None,
                },
            ],
        };
        let text = render_index(&index);
        let alpha_index = text.find("\"alpha\"").expect("alpha present");
        let zulu_index = text.find("\"zulu\"").expect("zulu present");
        assert!(alpha_index < zulu_index);
        assert!(text.ends_with('\n'));
    }
}
