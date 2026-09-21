// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::collections::HashSet;

use crate::app::state::Step3ItemState;
use crate::ui::step3::blocks;
use eframe::egui;

pub fn apply_row_selection(
    selected: &mut Vec<usize>,
    anchor: &mut Option<usize>,
    items: &[Step3ItemState],
    visible_indices: &[usize],
    idx: usize,
    modifiers: egui::Modifiers,
) {
    if modifiers.shift {
        selected.clear();
        let start = *anchor.get_or_insert(idx);
        let start_pos = visible_indices.iter().position(|v| *v == start);
        let end_pos = visible_indices.iter().position(|v| *v == idx);
        if let (Some(a), Some(b)) = (start_pos, end_pos) {
            let (from, to) = if a <= b { (a, b) } else { (b, a) };
            *selected = range_selection(items, visible_indices, from, to, [start, idx]);
        } else {
            selected.push(idx);
        }
        selected.sort_unstable();
        selected.dedup();
    } else if modifiers.ctrl {
        if let Some(pos) = selected.iter().position(|v| *v == idx) {
            selected.remove(pos);
        } else {
            selected.push(idx);
            selected.sort_unstable();
            selected.dedup();
        }
        *anchor = Some(idx);
    } else {
        selected.clear();
        selected.push(idx);
        *anchor = Some(idx);
    }
}

fn range_selection(
    items: &[Step3ItemState],
    visible_indices: &[usize],
    from: usize,
    to: usize,
    endpoints: [usize; 2],
) -> Vec<usize> {
    let range = &visible_indices[from..=to];
    let in_range: HashSet<usize> = range.iter().copied().collect();
    let visible: HashSet<usize> = visible_indices.iter().copied().collect();
    let mut out = Vec::with_capacity(range.len());
    for &row in range {
        let Some(item) = items.get(row) else {
            continue;
        };
        if !item.is_parent {
            out.push(row);
            continue;
        }
        let block = blocks::block_indices(items, row);
        let mut children = block.iter().copied().filter(|member| *member != row);
        let whole_mod_covered = endpoints.contains(&row)
            || children.all(|child| in_range.contains(&child) || !visible.contains(&child));
        if whole_mod_covered {
            out.extend(block);
        }
    }
    out
}

pub mod component_uncheck {
    pub(crate) use crate::ui::step3::service_component_uncheck_step3::*;
}

pub mod prompt_actions {
    pub(crate) use crate::ui::step3::service_prompt_actions_step3::*;
}

pub mod drag_ops {
    pub(crate) use crate::ui::step3::service_drag_ops_step3::*;
}

#[cfg(test)]
mod tests {
    use super::apply_row_selection;
    use crate::app::state::Step3ItemState;
    use eframe::egui;

    const SHIFT: egui::Modifiers = egui::Modifiers {
        alt: false,
        ctrl: false,
        shift: true,
        mac_cmd: false,
        command: false,
    };
    const CTRL: egui::Modifiers = egui::Modifiers {
        alt: false,
        ctrl: true,
        shift: false,
        mac_cmd: false,
        command: false,
    };

    fn row(mod_name: &str, component_id: &str, is_parent: bool) -> Step3ItemState {
        Step3ItemState {
            tp_file: format!("{mod_name}.tp2"),
            component_id: component_id.to_string(),
            mod_name: mod_name.to_string(),
            component_label: String::new(),
            raw_line: String::new(),
            prompt_summary: None,
            prompt_events: Vec::new(),
            selected_order: 0,
            block_id: format!("{mod_name}::block0"),
            is_parent,
            parent_placeholder: false,
        }
    }

    fn three_mods_of_two() -> Vec<Step3ItemState> {
        let mut items = Vec::new();
        for mod_name in ["A", "B", "C"] {
            items.push(row(mod_name, "__PARENT__", true));
            items.push(row(mod_name, "1", false));
            items.push(row(mod_name, "2", false));
        }
        items
    }

    fn every_row_visible() -> Vec<usize> {
        (0..9).collect()
    }

    fn with_mod_b_collapsed() -> Vec<usize> {
        vec![0, 1, 2, 3, 6, 7, 8]
    }

    #[test]
    fn shift_click_across_mods_takes_the_components_and_only_fully_covered_headers() {
        let items = three_mods_of_two();
        let mut selected = vec![2];
        let mut anchor = Some(2);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            7,
            SHIFT,
        );

        assert_eq!(selected, vec![2, 3, 4, 5, 7]);
        assert_eq!(anchor, Some(2));
    }

    #[test]
    fn shift_click_upward_selects_the_same_rows() {
        let items = three_mods_of_two();
        let mut selected = vec![7];
        let mut anchor = Some(7);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            2,
            SHIFT,
        );

        assert_eq!(selected, vec![2, 3, 4, 5, 7]);
        assert_eq!(anchor, Some(7));
    }

    #[test]
    fn shift_click_inside_one_mod_selects_that_run_without_its_header() {
        let items = three_mods_of_two();
        let mut selected = vec![1];
        let mut anchor = Some(1);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            2,
            SHIFT,
        );

        assert_eq!(selected, vec![1, 2]);
    }

    #[test]
    fn a_collapsed_mod_inside_the_range_is_taken_whole_with_its_hidden_components() {
        let items = three_mods_of_two();
        let mut selected = vec![1];
        let mut anchor = Some(1);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &with_mod_b_collapsed(),
            7,
            SHIFT,
        );

        assert_eq!(selected, vec![1, 2, 3, 4, 5, 7]);
    }

    #[test]
    fn shift_clicking_a_header_takes_that_whole_mod() {
        let items = three_mods_of_two();
        let mut selected = vec![2];
        let mut anchor = Some(2);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            6,
            SHIFT,
        );

        assert_eq!(selected, vec![2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn a_second_shift_click_replaces_the_range_from_the_same_anchor() {
        let items = three_mods_of_two();
        let mut selected = vec![2];
        let mut anchor = Some(2);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            7,
            SHIFT,
        );
        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            4,
            SHIFT,
        );

        assert_eq!(selected, vec![2, 4]);
        assert_eq!(anchor, Some(2));
    }

    #[test]
    fn a_first_shift_click_sets_the_anchor_so_the_next_one_makes_a_range() {
        let items = three_mods_of_two();
        let mut selected = Vec::new();
        let mut anchor = None;

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            5,
            SHIFT,
        );

        assert_eq!(selected, vec![5]);
        assert_eq!(anchor, Some(5));

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            7,
            SHIFT,
        );

        assert_eq!(selected, vec![5, 7]);
    }

    #[test]
    fn a_header_anchor_takes_its_whole_mod_into_the_range() {
        let items = three_mods_of_two();
        let mut selected = vec![3];
        let mut anchor = Some(3);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            7,
            SHIFT,
        );

        assert_eq!(selected, vec![3, 4, 5, 7]);
    }

    #[test]
    fn shift_click_from_a_hidden_anchor_selects_the_clicked_row() {
        let items = three_mods_of_two();
        let mut selected = vec![4];
        let mut anchor = Some(4);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &with_mod_b_collapsed(),
            7,
            SHIFT,
        );

        assert_eq!(selected, vec![7]);
    }

    #[test]
    fn ctrl_click_after_a_range_adds_and_removes_single_rows() {
        let items = three_mods_of_two();
        let mut selected = vec![2];
        let mut anchor = Some(2);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            4,
            SHIFT,
        );
        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            8,
            CTRL,
        );
        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            4,
            CTRL,
        );

        assert_eq!(selected, vec![2, 8]);
        assert_eq!(anchor, Some(4));
    }

    #[test]
    fn plain_click_replaces_the_selection_and_moves_the_anchor() {
        let items = three_mods_of_two();
        let mut selected = vec![1, 2, 3];
        let mut anchor = Some(1);

        apply_row_selection(
            &mut selected,
            &mut anchor,
            &items,
            &every_row_visible(),
            6,
            egui::Modifiers::NONE,
        );

        assert_eq!(selected, vec![6]);
        assert_eq!(anchor, Some(6));
    }
}
