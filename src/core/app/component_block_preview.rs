// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::fs;

const DETAILS_COMPONENT_PREVIEW_MAX_LINES: usize = 48;
const DETAILS_COMPONENT_PREVIEW_MAX_CHARS: usize = 3_000;
const DETAILS_COMPONENT_PREVIEW_SOFT_EXTEND_LINES: usize = 12;

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
            .find_map(|entry| parse_designated_id(&entry.to_ascii_uppercase()))
            .unwrap_or_else(|| position.to_string());
        if block_id == wanted {
            let display_start = component_block_start(&lines, start);
            let display_end = component_block_end(&lines, display_start, end);
            let mut preview = lines[display_start..display_end].join("\n");
            if display_end < end {
                preview.push_str("\n...");
            }
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
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        if is_component_begin_line(line) {
            starts.push(index);
        }
    }
    starts
}

fn is_component_begin_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed
        .get(0..5)
        .filter(|prefix| prefix.eq_ignore_ascii_case("BEGIN"))
        .map(|_| &trimmed[5..])
    else {
        return false;
    };
    let Some(first_ws) = rest.chars().next() else {
        return false;
    };
    if !first_ws.is_whitespace() {
        return false;
    }
    let after_ws = rest.trim_start();
    matches!(after_ws.chars().next(), Some('~' | '"' | '@' | '%'))
}

fn component_block_start(lines: &[&str], block_start: usize) -> usize {
    let mut start = block_start;
    let mut saw_comment = false;

    while start > 0 {
        let prev = lines[start - 1].trim();
        if prev.is_empty() {
            start -= 1;
            continue;
        }
        if is_component_header_comment(prev) {
            saw_comment = true;
            start -= 1;
            continue;
        }
        break;
    }

    if saw_comment { start } else { block_start }
}

fn component_block_end(lines: &[&str], block_start: usize, block_end: usize) -> usize {
    let mut end = block_start;
    let mut char_count = 0usize;

    while end < block_end {
        char_count += lines[end].len() + 1;
        end += 1;
        if end - block_start >= DETAILS_COMPONENT_PREVIEW_MAX_LINES
            || char_count >= DETAILS_COMPONENT_PREVIEW_MAX_CHARS
        {
            break;
        }
    }

    if end >= block_end {
        while end > block_start && lines[end - 1].trim().is_empty() {
            end -= 1;
        }
        return end;
    }

    let mut extended = end;
    let soft_limit = (end + DETAILS_COMPONENT_PREVIEW_SOFT_EXTEND_LINES).min(block_end);
    while extended < soft_limit {
        let line = lines[extended].trim();
        extended += 1;
        if line.is_empty() {
            end = extended;
            break;
        }
        if is_component_section_banner_line(line) {
            end = extended.saturating_sub(1);
            break;
        }
    }

    while end > block_start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }

    if end == block_start {
        let mut fallback = block_start + 1;
        while fallback < block_end {
            let line = lines[fallback].trim();
            if line.is_empty()
                || is_component_header_comment(line)
                || is_component_header_line(line)
            {
                fallback += 1;
                continue;
            }
            break;
        }
        while fallback > block_start && lines[fallback - 1].trim().is_empty() {
            fallback -= 1;
        }
        return fallback.max((block_start + 1).min(block_end));
    }

    end
}

fn is_component_header_comment(line: &str) -> bool {
    line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with('*')
        || line.starts_with("*/")
}

fn is_component_section_banner_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let slash_count = trimmed.chars().filter(|&ch| ch == '/').count();
    let backslash_count = trimmed.chars().filter(|&ch| ch == '\\').count();
    slash_count >= 6 && backslash_count >= 6
}

fn is_component_header_line(line: &str) -> bool {
    let upper = line.trim_start().to_ascii_uppercase();
    upper.starts_with("REQUIRE_")
        || upper.starts_with("FORBID_")
        || upper.starts_with("DESIGNATED")
        || upper.starts_with("LABEL")
        || upper.starts_with("GROUP")
        || upper.starts_with("SUBCOMPONENT")
        || upper.starts_with("VERSION")
}

fn parse_designated_id(upper_line: &str) -> Option<String> {
    if upper_line.trim_start().starts_with("//") {
        return None;
    }
    let index = upper_line.find("DESIGNATED")?;
    let tail = upper_line[index + "DESIGNATED".len()..].trim_start();
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        None
    } else {
        let normalized = digits.trim_start_matches('0');
        if normalized.is_empty() {
            Some("0".to_string())
        } else {
            Some(normalized.to_string())
        }
    }
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
}
