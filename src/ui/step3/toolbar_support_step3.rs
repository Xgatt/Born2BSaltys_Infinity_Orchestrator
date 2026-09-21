// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

pub(crate) use crate::app::step3_toolbar::{
    Step3ToolbarSummary, build_toolbar_summary, open_toolbar_issue_popup, tab_has_conflict,
};

use crate::app::state::WizardState;
use crate::ui::step3::move_selection_step3::keep_selection_across;
use crate::ui::step5::service_diagnostics_support_step5::export_diagnostics;

pub(crate) fn export_diagnostics_from_step3(
    state: &mut WizardState,
    dev_mode: bool,
    exe_fingerprint: &str,
) {
    match export_diagnostics(state, None, dev_mode, exe_fingerprint) {
        Ok(path) => {
            state.step5.last_status_text = format!("Diagnostics exported: {}", path.display());
        }
        Err(err) => {
            state.step5.last_status_text = format!("Diagnostics export failed: {err}");
        }
    }
}

pub(crate) fn expand_all_active(state: &mut WizardState) {
    let (_, selected, _, _, _, anchor, _, _, _, _, collapsed_blocks, _, _, _, _) =
        crate::ui::step3::state_step3::active_list_mut(state);
    crate::app::step3_history::expand_all(collapsed_blocks);
    selected.clear();
    *anchor = None;
}

pub(crate) fn collapse_all_active(state: &mut WizardState) {
    let (items, selected, _, _, _, anchor, _, _, _, _, collapsed_blocks, _, _, _, _) =
        crate::ui::step3::state_step3::active_list_mut(state);
    crate::app::step3_history::collapse_all(items, collapsed_blocks);
    selected.clear();
    *anchor = None;
}

pub(crate) fn redo_active(state: &mut WizardState) {
    let (items, selected, _, _, _, anchor, _, _, _, _, _, _, _, undo_stack, redo_stack) =
        crate::ui::step3::state_step3::active_list_mut(state);
    keep_selection_across(items, selected, anchor, |items| {
        crate::app::step3_history::redo(items, undo_stack, redo_stack);
    });
}

pub(crate) fn undo_active(state: &mut WizardState) {
    let (items, selected, _, _, _, anchor, _, _, _, _, _, _, _, undo_stack, redo_stack) =
        crate::ui::step3::state_step3::active_list_mut(state);
    keep_selection_across(items, selected, anchor, |items| {
        crate::app::step3_history::undo(items, undo_stack, redo_stack);
    });
}

#[cfg(test)]
mod tests {
    use super::{collapse_all_active, expand_all_active, redo_active, undo_active};
    use crate::app::state::{Step3ItemState, WizardState};
    use crate::app::step3_history;
    use crate::ui::step3::state_step3::active_list_mut;

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

    fn state_with_two_mods_and_a_selection(selection: &[usize]) -> WizardState {
        let mut state = WizardState::default();
        let (items, selected, _, _, _, anchor, ..) = active_list_mut(&mut state);
        *items = vec![
            row("A", "__PARENT__", true),
            row("A", "1", false),
            row("B", "__PARENT__", true),
            row("B", "1", false),
            row("B", "2", false),
        ];
        *selected = selection.to_vec();
        *anchor = selection.first().copied();
        state
    }

    fn selection_and_anchor(state: &mut WizardState) -> (Vec<usize>, Option<usize>) {
        let (_, selected, _, _, _, anchor, ..) = active_list_mut(state);
        (selected.clone(), *anchor)
    }

    #[test]
    fn collapse_all_clears_the_selection() {
        let mut state = state_with_two_mods_and_a_selection(&[1, 3]);

        collapse_all_active(&mut state);

        assert_eq!(selection_and_anchor(&mut state), (Vec::new(), None));
    }

    #[test]
    fn expand_all_clears_the_selection() {
        let mut state = state_with_two_mods_and_a_selection(&[1, 3]);

        expand_all_active(&mut state);

        assert_eq!(selection_and_anchor(&mut state), (Vec::new(), None));
    }

    #[test]
    fn undo_and_redo_keep_the_same_rows_selected() {
        let mut state = state_with_two_mods_and_a_selection(&[2, 3, 4]);
        {
            let (items, selected, .., undo_stack, redo_stack) = active_list_mut(&mut state);
            step3_history::push_undo_snapshot(items, undo_stack, redo_stack);
            let mod_b: Vec<Step3ItemState> = items.drain(2..).collect();
            items.splice(0..0, mod_b);
            *selected = vec![0, 1, 2];
        }

        undo_active(&mut state);
        assert_eq!(selection_and_anchor(&mut state), (vec![2, 3, 4], Some(2)));

        redo_active(&mut state);
        assert_eq!(selection_and_anchor(&mut state), (vec![0, 1, 2], Some(0)));
    }
}
