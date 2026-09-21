// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

pub mod constraints {
    use crate::app::state::Step3ItemState;

    #[must_use]
    pub fn resolve_insert_at(
        remaining: &[Step3ItemState],
        insert_at: usize,
        moving: &[Step3ItemState],
        locked_blocks: &[String],
    ) -> usize {
        let at = insert_at.min(remaining.len());
        if moving.is_empty() || at == 0 || at >= remaining.len() || remaining[at].is_parent {
            return at;
        }
        if joins_its_own_mod(remaining, at, moving) {
            return at;
        }
        if remaining[at - 1].is_parent {
            return at - 1;
        }
        let block_id = remaining[at].block_id.clone();
        if locked_blocks.contains(&block_id) {
            return nearer_block_edge(remaining, at, &block_id);
        }
        at
    }

    fn joins_its_own_mod(
        remaining: &[Step3ItemState],
        at: usize,
        moving: &[Step3ItemState],
    ) -> bool {
        if moving.iter().any(|item| item.is_parent) {
            return false;
        }
        let Some(first) = moving.first() else {
            return false;
        };
        let key = mod_key(first);
        if moving.iter().any(|item| mod_key(item) != key) {
            return false;
        }
        key == mod_key(&remaining[at - 1])
    }

    fn nearer_block_edge(remaining: &[Step3ItemState], at: usize, block_id: &str) -> usize {
        let mut start = at;
        while start > 0 && remaining[start - 1].block_id == block_id {
            start -= 1;
        }
        let mut end = at;
        while end < remaining.len() && remaining[end].block_id == block_id {
            end += 1;
        }
        if at - start <= end - at { start } else { end }
    }

    #[must_use]
    fn mod_key(item: &Step3ItemState) -> String {
        format!(
            "{}::{}",
            item.tp_file.to_ascii_uppercase(),
            item.mod_name.to_ascii_uppercase()
        )
    }
}

pub mod math {
    #[must_use]
    pub fn compute_desired_block_start(
        pointer_y: f32,
        list_top_y: f32,
        row_pitch: f32,
        grab_offset: f32,
        grab_pos_in_block: usize,
        n: usize,
        k: usize,
    ) -> usize {
        let row_pitch = f64::from(row_pitch.max(1.0));
        let desired_grabbed_row =
            ((f64::from(pointer_y) - f64::from(list_top_y) - f64::from(grab_offset)) / row_pitch)
                .floor();
        let desired_start =
            desired_grabbed_row - f64::from(u32::try_from(grab_pos_in_block).unwrap_or(u32::MAX));
        let max_start = n.saturating_sub(k);

        let mut start = 0usize;
        while start < max_start {
            let next = start + 1;
            let next_as_float = f64::from(u32::try_from(next).unwrap_or(u32::MAX));
            if next_as_float > desired_start {
                break;
            }
            start = next;
        }
        start
    }
}

pub mod slots {
    use eframe::egui;

    use crate::app::state::Step3ItemState;

    #[must_use]
    pub fn visible_slot_to_insert_at(
        items: &[Step3ItemState],
        block: &[usize],
        visible_rows: &[(usize, egui::Rect)],
        target_visible_slot: usize,
        remaining_len: usize,
    ) -> usize {
        let remaining_full_indices: Vec<usize> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, _)| {
                if block.contains(&idx) {
                    None
                } else {
                    Some(idx)
                }
            })
            .collect();
        let visible_remaining_full: Vec<usize> = visible_rows
            .iter()
            .filter_map(|(idx, _)| {
                if block.contains(idx) {
                    None
                } else {
                    Some(*idx)
                }
            })
            .collect();
        if target_visible_slot >= visible_remaining_full.len() {
            return remaining_len;
        }
        let target_full = visible_remaining_full[target_visible_slot];
        remaining_full_indices
            .iter()
            .position(|idx| *idx == target_full)
            .unwrap_or(remaining_len)
    }
}

pub use constraints::resolve_insert_at;
pub use math::compute_desired_block_start;
pub use slots::visible_slot_to_insert_at;

#[cfg(test)]
mod tests {
    use super::compute_desired_block_start;

    #[test]
    fn start_is_clamped_to_valid_range() {
        let n = 10;
        let k = 4;
        let s1 = compute_desired_block_start(-1000.0, 100.0, 24.0, 5.0, 1, n, k);
        let s2 = compute_desired_block_start(99999.0, 100.0, 24.0, 5.0, 1, n, k);
        assert_eq!(s1, 0);
        assert_eq!(s2, n - k);
    }

    mod resolve_insert_at {
        use super::super::resolve_insert_at;
        use crate::app::state::Step3ItemState;

        fn row(
            mod_name: &str,
            component_id: &str,
            is_parent: bool,
            block_id: &str,
        ) -> Step3ItemState {
            Step3ItemState {
                tp_file: format!("{mod_name}.tp2"),
                component_id: component_id.to_string(),
                mod_name: mod_name.to_string(),
                component_label: String::new(),
                raw_line: String::new(),
                prompt_summary: None,
                prompt_events: Vec::new(),
                selected_order: 0,
                block_id: block_id.to_string(),
                is_parent,
                parent_placeholder: false,
            }
        }

        fn default_remaining() -> Vec<Step3ItemState> {
            vec![
                row("B", "__PARENT__", true, "B::b0"),
                row("B", "1", false, "B::b0"),
                row("B", "2", false, "B::b0"),
                row("B", "3", false, "B::b0"),
                row("C", "__PARENT__", true, "C::b0"),
                row("C", "1", false, "C::b0"),
            ]
        }

        #[test]
        fn foreign_components_land_between_two_components_of_another_mod() {
            let remaining = default_remaining();
            let moving = vec![row("A", "1", false, "A::b0")];
            assert_eq!(resolve_insert_at(&remaining, 2, &moving, &[]), 2);
            assert_eq!(resolve_insert_at(&remaining, 3, &moving, &[]), 3);
        }

        #[test]
        fn a_whole_mod_lands_between_two_components_of_another_mod() {
            let remaining = default_remaining();
            let moving = vec![
                row("A", "__PARENT__", true, "A::b0"),
                row("A", "1", false, "A::b0"),
            ];
            assert_eq!(resolve_insert_at(&remaining, 2, &moving, &[]), 2);
        }

        #[test]
        fn a_mixed_selection_lands_between_two_components_of_another_mod() {
            let remaining = default_remaining();
            let moving = vec![row("A", "1", false, "A::b0"), row("D", "1", false, "D::b0")];
            assert_eq!(resolve_insert_at(&remaining, 3, &moving, &[]), 3);
        }

        #[test]
        fn directly_under_a_foreign_header_snaps_above_it() {
            let remaining = default_remaining();
            let moving = vec![row("A", "1", false, "A::b0")];
            assert_eq!(resolve_insert_at(&remaining, 1, &moving, &[]), 0);

            let moving = vec![
                row("A", "__PARENT__", true, "A::b0"),
                row("A", "1", false, "A::b0"),
            ];
            assert_eq!(resolve_insert_at(&remaining, 5, &moving, &[]), 4);
        }

        #[test]
        fn a_locked_block_keeps_the_snap_to_its_nearer_edge() {
            let remaining = default_remaining();
            let moving = vec![row("A", "1", false, "A::b0")];
            let locked = vec!["B::b0".to_string()];
            assert_eq!(resolve_insert_at(&remaining, 2, &moving, &locked), 0);
            assert_eq!(resolve_insert_at(&remaining, 3, &moving, &locked), 4);

            let locked = vec!["C::b0".to_string()];
            assert_eq!(resolve_insert_at(&remaining, 2, &moving, &locked), 2);
        }

        #[test]
        fn components_still_join_their_own_mod_anywhere() {
            let remaining = default_remaining();
            let moving = vec![row("B", "9", false, "B::b0")];
            assert_eq!(resolve_insert_at(&remaining, 1, &moving, &[]), 1);
            assert_eq!(resolve_insert_at(&remaining, 2, &moving, &[]), 2);

            let locked = vec!["B::b0".to_string()];
            assert_eq!(resolve_insert_at(&remaining, 1, &moving, &locked), 1);
            assert_eq!(resolve_insert_at(&remaining, 2, &moving, &locked), 2);
        }

        #[test]
        fn edges_and_header_boundaries_are_left_alone() {
            let remaining = default_remaining();
            let moving = vec![row("A", "1", false, "A::b0")];
            assert_eq!(resolve_insert_at(&remaining, 0, &moving, &[]), 0);
            assert_eq!(resolve_insert_at(&remaining, 4, &moving, &[]), 4);
            assert_eq!(resolve_insert_at(&remaining, 6, &moving, &[]), 6);
            assert_eq!(resolve_insert_at(&remaining, 99, &moving, &[]), 6);
            assert_eq!(resolve_insert_at(&remaining, 3, &[], &[]), 3);
        }
    }
}
