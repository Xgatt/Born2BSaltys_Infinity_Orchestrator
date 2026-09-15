// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::path::{Path, PathBuf};

fn count_entries(index_text: &str) -> usize {
    serde_json::from_str::<serde_json::Value>(index_text)
        .ok()
        .and_then(|value| {
            value
                .get("entries")
                .and_then(|entries| entries.as_array().map(Vec::len))
        })
        .unwrap_or(0)
}

fn run_build(root: &Path) {
    let report = bio::gallery_feed::folder::build_index(root);
    if !report.errors.is_empty() {
        for error in &report.errors {
            eprintln!("{error}");
        }
        std::process::exit(1);
    }
    let index_text = report.index_text.unwrap_or_default();
    let entry_count = count_entries(&index_text);
    if let Err(err) = std::fs::write(root.join("index.json"), &index_text) {
        eprintln!("failed to write index.json: {err}");
        std::process::exit(1);
    }
    println!("wrote {entry_count} entries");
}

fn run_check(root: &Path) {
    match bio::gallery_feed::folder::check_index(root) {
        Ok(count) => println!("ok: {count} entries"),
        Err(errors) => {
            for error in &errors {
                eprintln!("{error}");
            }
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match (args.get(1).map(String::as_str), args.get(2)) {
        (Some("build"), Some(root)) => run_build(&PathBuf::from(root)),
        (Some("check"), Some(root)) => run_check(&PathBuf::from(root)),
        _ => {
            eprintln!("gallery-index build <folder> | check <folder>");
            std::process::exit(2);
        }
    }
}
