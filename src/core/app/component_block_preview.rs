// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::fs;

use crate::app::tp2_component_begin::{component_begin_at, designated_id_in_code};

pub(crate) fn load_component_block_preview(tp2_path: &str, component_id: &str) -> Option<String> {
    if tp2_path.trim().is_empty() || component_id.trim().is_empty() {
        return None;
    }
    let tp2_text = decode_tp2_text(fs::read(tp2_path).ok()?);
    let lines: Vec<&str> = tp2_text.lines().collect();
    let wanted = normalize_component_id(component_id)?;
    let starts = component_block_starts(&lines);
    for (position, &start) in starts.iter().enumerate() {
        let end = starts.get(position + 1).copied().unwrap_or(lines.len());
        let block_id = lines[start..end]
            .iter()
            .find_map(|entry| designated_id_in_code(entry))
            .unwrap_or_else(|| position.to_string());
        if block_id == wanted {
            let display_start = component_block_start(&lines, start);
            let display_end = component_block_end(&lines, display_start, end);
            let preview = lines[display_start..display_end].join("\n");
            return Some(preview);
        }
    }
    None
}

fn decode_tp2_text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap_or_else(|err| {
        err.as_bytes()
            .iter()
            .map(|&byte| char::from(byte))
            .collect()
    })
}

fn normalize_component_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let stripped = trimmed.trim_start_matches('0');
    Some(if stripped.is_empty() {
        "0".to_string()
    } else {
        stripped.to_string()
    })
}

fn component_block_starts(lines: &[&str]) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut in_block_comment = false;
    for (index, &line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if in_block_comment {
            if let Some(pos) = trimmed.find("*/") {
                in_block_comment = false;
                let remainder = &trimmed[pos + 2..];
                if component_begin_at(remainder, lines, index).is_some() {
                    starts.push(index);
                }
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if let Some(pos) = trimmed.find("*/") {
                let remainder = &trimmed[pos + 2..];
                if component_begin_at(remainder, lines, index).is_some() {
                    starts.push(index);
                }
            } else {
                in_block_comment = true;
            }
            continue;
        }
        if component_begin_at(line, lines, index).is_some() {
            starts.push(index);
        }
    }
    starts
}

fn component_block_start(lines: &[&str], block_start: usize) -> usize {
    let mut start = block_start;
    while start > 0 {
        let prev = lines[start - 1].trim();
        if prev.is_empty() {
            break;
        }
        if is_component_header_comment(prev) {
            start -= 1;
            continue;
        }
        break;
    }
    start
}

fn component_block_end(lines: &[&str], block_start: usize, block_end: usize) -> usize {
    let mut end = block_end;
    while end > block_start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    end
}

fn is_component_header_comment(line: &str) -> bool {
    line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with('*')
        || line.starts_with("*/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    struct TestFixture {
        dir: PathBuf,
        tp2_path: PathBuf,
    }

    impl TestFixture {
        fn new(content: &str) -> Self {
            Self::from_bytes(content.as_bytes())
        }

        fn from_bytes(content: &[u8]) -> Self {
            let unique = NEXT_ID.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!("bio_blockpreview_test_{unique}"));
            fs::create_dir_all(&dir).expect("create fixture dir");
            let tp2_path = dir.join("setup-fixture.tp2");
            fs::write(&tp2_path, content).expect("write fixture tp2");
            Self { dir, tp2_path }
        }

        fn tp2_path_str(&self) -> String {
            self.tp2_path.to_string_lossy().into_owned()
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn non_utf8_tp2_still_resolves_blocks() {
        let fixture = TestFixture::from_bytes(b"BEGIN ~Caf\xE9~\nbody a\n\nBEGIN ~B~\nbody b\n");
        let path = fixture.tp2_path_str();
        assert!(
            load_component_block_preview(&path, "1")
                .unwrap()
                .contains("BEGIN ~B~")
        );
        assert!(
            load_component_block_preview(&path, "0")
                .unwrap()
                .contains("BEGIN ~Café~")
        );
    }

    #[test]
    fn undesignated_components_are_numbered_by_position() {
        let fixture =
            TestFixture::new("BEGIN ~A~\nbody a\n\nBEGIN ~B~\nbody b\n\nBEGIN ~C~\nbody c\n");
        let path = fixture.tp2_path_str();
        assert!(
            load_component_block_preview(&path, "1")
                .unwrap()
                .contains("BEGIN ~B~")
        );
        assert!(
            load_component_block_preview(&path, "2")
                .unwrap()
                .contains("BEGIN ~C~")
        );
        assert!(load_component_block_preview(&path, "3").is_none());
    }

    #[test]
    fn designated_overrides_only_its_own_block() {
        let fixture = TestFixture::new(
            "BEGIN ~A~\nbody a\n\nBEGIN ~B~\nDESIGNATED 10\nbody b\n\nBEGIN ~C~\nbody c\n",
        );
        let path = fixture.tp2_path_str();
        assert!(
            load_component_block_preview(&path, "10")
                .unwrap()
                .contains("BEGIN ~B~")
        );
        assert!(
            load_component_block_preview(&path, "2")
                .unwrap()
                .contains("BEGIN ~C~")
        );
        assert!(load_component_block_preview(&path, "1").is_none());
    }

    #[test]
    fn bare_begin_inside_actions_is_not_a_component() {
        let fixture =
            TestFixture::new("BEGIN ~A~\nACTION_IF x BEGIN\nBEGIN   // comment\nBEGIN @2\nbody\n");
        let path = fixture.tp2_path_str();
        assert!(
            load_component_block_preview(&path, "1")
                .unwrap()
                .contains("BEGIN @2")
        );
        assert!(load_component_block_preview(&path, "2").is_none());
    }

    #[test]
    fn commented_out_begins_do_not_count() {
        let fixture = TestFixture::new(
            "// BEGIN ~X~\n/*\nBEGIN ~Y~\nsome comment body\n*/\nBEGIN ~Z~\nbody z\n",
        );
        let path = fixture.tp2_path_str();
        assert!(
            load_component_block_preview(&path, "0")
                .unwrap()
                .contains("BEGIN ~Z~")
        );
    }

    #[test]
    fn padded_ids_match() {
        let fixture = TestFixture::new("BEGIN ~A~\nDESIGNATED 007\nbody\n");
        let path = fixture.tp2_path_str();
        assert!(load_component_block_preview(&path, "7").is_some());
        assert!(load_component_block_preview(&path, "07").is_some());
    }

    #[test]
    fn at_label_begins_count_as_components() {
        let fixture =
            TestFixture::new("BEGIN @0\nbody0\n\nBEGIN @20\nbody20\n\nBEGIN @47\nbody47\n");
        let path = fixture.tp2_path_str();
        assert!(
            load_component_block_preview(&path, "1")
                .unwrap()
                .contains("BEGIN @20")
        );
    }

    #[test]
    fn label_on_next_line_counts_as_component_and_shifts_nothing() {
        let fixture = TestFixture::new(
            "BEGIN ~A~\nbodyA\n\nBEGIN ~B~\nbodyB\n\nBEGIN\n~C~\nbodyC\n\nBEGIN ~D~\nbodyD\n",
        );
        let path = fixture.tp2_path_str();
        let block = load_component_block_preview(&path, "3").unwrap();
        assert!(block.contains("BEGIN ~D~"));
        assert!(block.contains("bodyD"));
        let bare_block = load_component_block_preview(&path, "2").unwrap();
        assert!(bare_block.contains("bodyC"));
    }

    #[test]
    fn block_comment_closing_before_begin_on_same_line() {
        let fixture = TestFixture::new("/* header\n*/ BEGIN ~X~\nbody x\n");
        let path = fixture.tp2_path_str();
        assert!(
            load_component_block_preview(&path, "0")
                .unwrap()
                .contains("BEGIN ~X~")
        );
    }

    #[test]
    fn designated_in_trailing_comment_is_ignored() {
        let fixture = TestFixture::new("BEGIN ~A~ // DESIGNATED 99\nbody a\n\nBEGIN ~B~\nbody b\n");
        let path = fixture.tp2_path_str();
        assert!(load_component_block_preview(&path, "99").is_none());
        assert!(
            load_component_block_preview(&path, "0")
                .unwrap()
                .contains("BEGIN ~A~")
        );
    }

    #[test]
    fn two_hundred_line_component_is_returned_whole() {
        let body_lines: Vec<String> = (0..200).map(|n| format!("line{n}")).collect();
        let content = format!(
            "BEGIN ~Big~\n{}\n\nBEGIN ~Next~\nbody\n",
            body_lines.join("\n")
        );
        let fixture = TestFixture::new(&content);
        let path = fixture.tp2_path_str();
        let block = load_component_block_preview(&path, "0").unwrap();
        assert!(!block.contains("..."));
        assert!(block.contains("line0"));
        assert!(block.contains("line199"));
    }

    #[test]
    fn header_comments_absorbed_only_while_contiguous() {
        let fixture =
            TestFixture::new("// first comment\n\n// second comment\nBEGIN ~A~\nbody a\n");
        let path = fixture.tp2_path_str();
        let block = load_component_block_preview(&path, "0").unwrap();
        assert!(block.contains("// second comment"));
        assert!(!block.contains("// first comment"));
    }
}
