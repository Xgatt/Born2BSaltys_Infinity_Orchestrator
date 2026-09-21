// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::collections::HashSet;

use crate::app::state::Step3ItemState;
use crate::app::step3_history;
use crate::ui::step3::blocks;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoveSelectionTarget {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoveSelectionOutcome {
    Moved,
    RefusedLocked,
    NothingToMove,
}

pub(crate) struct MoveSelectionContext<'a> {
    pub items: &'a mut Vec<Step3ItemState>,
    pub selected: &'a mut Vec<usize>,
    pub anchor: &'a mut Option<usize>,
    pub clone_seq: &'a mut usize,
    pub locked_blocks: &'a [String],
    pub undo_stack: &'a mut Vec<Vec<Step3ItemState>>,
    pub redo_stack: &'a mut Vec<Vec<Step3ItemState>>,
}

#[must_use]
pub(crate) fn moving_set(
    items: &[Step3ItemState],
    selected: &[usize],
    clicked_idx: usize,
) -> Vec<usize> {
    let operands: Vec<usize> = if selected.contains(&clicked_idx)
        || header_of_a_fully_selected_mod(items, selected, clicked_idx)
    {
        selected
            .iter()
            .copied()
            .filter(|idx| *idx < items.len())
            .collect()
    } else {
        vec![clicked_idx]
    };

    let mut moving: HashSet<usize> = HashSet::new();
    for idx in operands {
        if items[idx].is_parent {
            for child_idx in blocks::block_indices(items, idx) {
                moving.insert(child_idx);
            }
        } else {
            moving.insert(idx);
        }
    }

    for (idx, item) in items.iter().enumerate() {
        if !item.is_parent {
            continue;
        }
        let block_indices = blocks::block_indices(items, idx);
        let children: Vec<usize> = block_indices
            .iter()
            .copied()
            .filter(|child_idx| !items[*child_idx].is_parent)
            .collect();
        let has_moving_child = children.iter().any(|child_idx| moving.contains(child_idx));
        let all_children_moving =
            !children.is_empty() && children.iter().all(|child_idx| moving.contains(child_idx));
        if has_moving_child && all_children_moving {
            moving.insert(idx);
        }
    }

    let mut moving: Vec<usize> = moving.into_iter().collect();
    moving.sort_unstable();
    moving
}

fn header_of_a_fully_selected_mod(
    items: &[Step3ItemState],
    selected: &[usize],
    clicked_idx: usize,
) -> bool {
    if !items.get(clicked_idx).is_some_and(|item| item.is_parent) {
        return false;
    }
    let mut children = blocks::block_indices(items, clicked_idx)
        .into_iter()
        .filter(|member| *member != clicked_idx)
        .peekable();
    children.peek().is_some() && children.all(|child| selected.contains(&child))
}

#[must_use]
pub(crate) fn is_locked(
    items: &[Step3ItemState],
    moving: &[usize],
    locked_blocks: &[String],
) -> bool {
    moving
        .iter()
        .any(|idx| locked_blocks.contains(&items[*idx].block_id))
}

fn already_at_target(len: usize, moving: &[usize], target: MoveSelectionTarget) -> bool {
    let n = moving.len();
    let expected: Vec<usize> = match target {
        MoveSelectionTarget::Top => (0..n).collect(),
        MoveSelectionTarget::Bottom => (len - n..len).collect(),
    };
    moving == expected.as_slice()
}

struct MovingIdentities {
    child_keys: HashSet<String>,
    parent_block_ids: HashSet<String>,
}

fn capture_identities(items: &[Step3ItemState], moving: &[usize]) -> MovingIdentities {
    let mut child_keys = HashSet::new();
    let mut parent_block_ids = HashSet::new();
    for idx in moving {
        let item = &items[*idx];
        if item.is_parent {
            parent_block_ids.insert(item.block_id.clone());
        } else {
            child_keys.insert(blocks::step3_item_key(item));
        }
    }
    MovingIdentities {
        child_keys,
        parent_block_ids,
    }
}

fn rebuild_items(items: &mut Vec<Step3ItemState>, moving: &[usize], target: MoveSelectionTarget) {
    let moving_set: HashSet<usize> = moving.iter().copied().collect();
    let mut remaining: Vec<Step3ItemState> = Vec::with_capacity(items.len() - moving.len());
    let mut moved_rows: Vec<Step3ItemState> = Vec::with_capacity(moving.len());
    for (idx, item) in items.drain(..).enumerate() {
        if moving_set.contains(&idx) {
            moved_rows.push(item);
        } else {
            remaining.push(item);
        }
    }
    *items = match target {
        MoveSelectionTarget::Top => {
            moved_rows.extend(remaining);
            moved_rows
        }
        MoveSelectionTarget::Bottom => {
            remaining.extend(moved_rows);
            remaining
        }
    };
}

fn recompute_selection(items: &[Step3ItemState], identities: &MovingIdentities) -> Vec<usize> {
    let mut selected: Vec<usize> = items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            let matches = if item.is_parent {
                identities.parent_block_ids.contains(&item.block_id)
            } else {
                identities
                    .child_keys
                    .contains(&blocks::step3_item_key(item))
            };
            matches.then_some(idx)
        })
        .collect();
    selected.sort_unstable();
    selected.dedup();
    selected
}

pub(crate) fn move_selection(
    ctx: &mut MoveSelectionContext<'_>,
    clicked_idx: usize,
    target: MoveSelectionTarget,
) -> MoveSelectionOutcome {
    if clicked_idx >= ctx.items.len() {
        return MoveSelectionOutcome::NothingToMove;
    }

    let moving = moving_set(ctx.items, ctx.selected, clicked_idx);

    if is_locked(ctx.items, &moving, ctx.locked_blocks) {
        return MoveSelectionOutcome::RefusedLocked;
    }

    if already_at_target(ctx.items.len(), &moving, target) {
        return MoveSelectionOutcome::NothingToMove;
    }

    step3_history::push_undo_snapshot(ctx.items, ctx.undo_stack, ctx.redo_stack);

    let identities = capture_identities(ctx.items, &moving);

    rebuild_items(ctx.items, &moving, target);

    let n = moving.len();
    let len = ctx.items.len();
    *ctx.selected = match target {
        MoveSelectionTarget::Top => (0..n).collect(),
        MoveSelectionTarget::Bottom => (len - n..len).collect(),
    };

    blocks::repair_orphan_children(ctx.items, ctx.selected, ctx.clone_seq);
    blocks::merge_adjacent_same_mod_blocks(ctx.items, ctx.selected);
    blocks::prune_empty_parent_blocks(ctx.items, ctx.selected);

    *ctx.selected = recompute_selection(ctx.items, &identities);
    *ctx.anchor = ctx.selected.first().copied();

    MoveSelectionOutcome::Moved
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parent(mod_name: &str, block_id: &str) -> Step3ItemState {
        Step3ItemState {
            tp_file: format!("{mod_name}.tp2"),
            component_id: "__PARENT__".to_string(),
            mod_name: mod_name.to_string(),
            component_label: String::new(),
            raw_line: String::new(),
            prompt_summary: None,
            prompt_events: Vec::new(),
            selected_order: usize::MAX,
            block_id: block_id.to_string(),
            is_parent: true,
            parent_placeholder: false,
        }
    }

    fn child(
        mod_name: &str,
        block_id: &str,
        component_id: &str,
        selected_order: usize,
    ) -> Step3ItemState {
        Step3ItemState {
            tp_file: format!("{mod_name}.tp2"),
            component_id: component_id.to_string(),
            mod_name: mod_name.to_string(),
            component_label: format!("Component {component_id}"),
            raw_line: String::new(),
            prompt_summary: None,
            prompt_events: Vec::new(),
            selected_order,
            block_id: block_id.to_string(),
            is_parent: false,
            parent_placeholder: false,
        }
    }

    fn run(
        items: &mut Vec<Step3ItemState>,
        selected: &mut Vec<usize>,
        locked_blocks: &[String],
        clicked_idx: usize,
        target: MoveSelectionTarget,
    ) -> (
        MoveSelectionOutcome,
        Vec<Vec<Step3ItemState>>,
        Vec<Vec<Step3ItemState>>,
    ) {
        let mut anchor: Option<usize> = None;
        let mut clone_seq = 0usize;
        let mut undo_stack: Vec<Vec<Step3ItemState>> = Vec::new();
        let mut redo_stack: Vec<Vec<Step3ItemState>> = Vec::new();
        let outcome = {
            let mut ctx = MoveSelectionContext {
                items,
                selected,
                anchor: &mut anchor,
                clone_seq: &mut clone_seq,
                locked_blocks,
                undo_stack: &mut undo_stack,
                redo_stack: &mut redo_stack,
            };
            move_selection(&mut ctx, clicked_idx, target)
        };
        (outcome, undo_stack, redo_stack)
    }

    #[test]
    fn whole_block_moves_to_top_with_its_children() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
        ];
        let mut selected = vec![2, 3];
        let (outcome, ..) = run(&mut items, &mut selected, &[], 2, MoveSelectionTarget::Top);
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        assert_eq!(items[0].mod_name, "B");
        assert_eq!(items[1].mod_name, "B");
        assert_eq!(items[2].mod_name, "A");
        assert_eq!(items[3].mod_name, "A");
    }

    #[test]
    fn whole_block_moves_to_bottom_with_its_children() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
        ];
        let mut selected = vec![0, 1];
        let (outcome, ..) = run(
            &mut items,
            &mut selected,
            &[],
            0,
            MoveSelectionTarget::Bottom,
        );
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        assert_eq!(items[0].mod_name, "B");
        assert_eq!(items[1].mod_name, "B");
        assert_eq!(items[2].mod_name, "A");
        assert_eq!(items[3].mod_name, "A");
    }

    #[test]
    fn selected_children_move_to_top_under_a_split_parent() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
            child("B", "B::b0", "2", 3),
            child("B", "B::b0", "3", 4),
        ];
        let mut selected = vec![3, 4];
        let (outcome, ..) = run(&mut items, &mut selected, &[], 3, MoveSelectionTarget::Top);
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        assert!(items[0].is_parent);
        assert_eq!(items[0].mod_name, "B");
        assert!(items[0].block_id.contains("::split"));
        assert!(items[0].parent_placeholder);
        assert!(!items[1].is_parent);
        assert_eq!(items[1].component_id, "1");
        assert!(!items[2].is_parent);
        assert_eq!(items[2].component_id, "2");
        let remaining_block: Vec<&Step3ItemState> = items
            .iter()
            .filter(|item| !item.is_parent && item.mod_name == "B" && item.component_id == "3")
            .collect();
        assert_eq!(remaining_block.len(), 1);
        assert_eq!(remaining_block[0].block_id, "B::b0");
    }

    #[test]
    fn every_child_selected_takes_the_parent_along() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
            child("B", "B::b0", "2", 3),
        ];
        let mut selected = vec![3, 4];
        let (outcome, ..) = run(&mut items, &mut selected, &[], 3, MoveSelectionTarget::Top);
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        assert!(items[0].is_parent);
        assert!(!items[0].parent_placeholder);
        assert_eq!(items[0].mod_name, "B");
        assert_eq!(
            items
                .iter()
                .filter(|item| item.is_parent && item.mod_name == "B")
                .count(),
            1
        );
        assert!(
            !items
                .iter()
                .any(|item| item.is_parent && item.parent_placeholder && item.mod_name == "B")
        );
    }

    #[test]
    fn child_of_the_last_block_moved_to_bottom_stays_in_its_block() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
            child("B", "B::b0", "2", 3),
        ];
        let mut selected = vec![3];
        let (outcome, ..) = run(
            &mut items,
            &mut selected,
            &[],
            3,
            MoveSelectionTarget::Bottom,
        );
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        assert_eq!(items.len(), 5);
        assert!(items[2].is_parent);
        assert_eq!(items[2].mod_name, "B");
        assert_eq!(items[3].component_id, "2");
        assert_eq!(items[4].component_id, "1");
        assert_eq!(items[3].block_id, items[4].block_id);
        assert_eq!(items[3].block_id, "B::b0");
    }

    #[test]
    fn relative_order_follows_the_list_not_the_click_order() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            child("A", "A::b0", "2", 2),
            parent("C", "C::b0"),
            child("C", "C::b0", "1", 3),
            child("C", "C::b0", "2", 4),
        ];
        let mut selected = vec![4, 1];
        let (outcome, ..) = run(&mut items, &mut selected, &[], 4, MoveSelectionTarget::Top);
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        let a_pos = items
            .iter()
            .position(|item| item.mod_name == "A" && !item.is_parent)
            .unwrap();
        let c_pos = items
            .iter()
            .position(|item| item.mod_name == "C" && !item.is_parent)
            .unwrap();
        assert!(a_pos < c_pos);
    }

    #[test]
    fn locked_block_refuses_the_whole_move() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
        ];
        let before = items.clone();
        let mut selected = vec![1, 3];
        let (outcome, undo_stack, _) = run(
            &mut items,
            &mut selected,
            &["A::b0".to_string()],
            1,
            MoveSelectionTarget::Top,
        );
        assert_eq!(outcome, MoveSelectionOutcome::RefusedLocked);
        assert_eq!(items, before);
        assert!(undo_stack.is_empty());
    }

    #[test]
    fn row_outside_the_selection_moves_alone() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
            child("B", "B::b0", "2", 3),
            child("B", "B::b0", "3", 4),
        ];
        let mut selected = vec![1];
        let (outcome, ..) = run(
            &mut items,
            &mut selected,
            &[],
            3,
            MoveSelectionTarget::Bottom,
        );
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        assert_eq!(selected, vec![5]);
        assert_eq!(items[5].component_id, "1");
    }

    #[test]
    fn already_at_the_top_is_nothing_to_move() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
        ];
        let mut selected = vec![0, 1];
        let (outcome, undo_stack, _) =
            run(&mut items, &mut selected, &[], 0, MoveSelectionTarget::Top);
        assert_eq!(outcome, MoveSelectionOutcome::NothingToMove);
        assert!(undo_stack.is_empty());

        let mut items2 = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
        ];
        let mut selected2 = vec![2, 3];
        let (outcome2, undo_stack2, _) = run(
            &mut items2,
            &mut selected2,
            &[],
            2,
            MoveSelectionTarget::Bottom,
        );
        assert_eq!(outcome2, MoveSelectionOutcome::NothingToMove);
        assert!(undo_stack2.is_empty());
    }

    #[test]
    fn one_undo_snapshot_per_move() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
        ];
        let before = items.clone();
        let mut selected = vec![2, 3];
        let mut anchor: Option<usize> = None;
        let mut clone_seq = 0usize;
        let mut undo_stack: Vec<Vec<Step3ItemState>> = Vec::new();
        let mut redo_stack: Vec<Vec<Step3ItemState>> = vec![before.clone()];
        let outcome = {
            let mut ctx = MoveSelectionContext {
                items: &mut items,
                selected: &mut selected,
                anchor: &mut anchor,
                clone_seq: &mut clone_seq,
                locked_blocks: &[],
                undo_stack: &mut undo_stack,
                redo_stack: &mut redo_stack,
            };
            move_selection(&mut ctx, 2, MoveSelectionTarget::Top)
        };
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        assert_eq!(undo_stack.len(), 1);
        assert_eq!(undo_stack[0], before);
        assert!(redo_stack.is_empty());
    }

    #[test]
    fn moved_rows_stay_selected() {
        let mut items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 2),
            child("B", "B::b0", "2", 3),
            child("B", "B::b0", "3", 4),
        ];
        let mut selected = vec![3, 4];
        let (outcome, ..) = run(&mut items, &mut selected, &[], 3, MoveSelectionTarget::Top);
        assert_eq!(outcome, MoveSelectionOutcome::Moved);
        let moved_keys: HashSet<String> = vec![
            blocks::step3_item_key(&child("B", "B::b0", "1", 2)),
            blocks::step3_item_key(&child("B", "B::b0", "2", 3)),
        ]
        .into_iter()
        .collect();
        assert_eq!(selected.len(), 2);
        for idx in &selected {
            let item = &items[*idx];
            assert!(!item.is_parent);
            assert!(moved_keys.contains(&blocks::step3_item_key(item)));
        }
    }

    #[test]
    fn grabbing_the_header_of_a_fully_selected_mod_carries_the_whole_selection() {
        let items = vec![
            parent("A", "A::b0"),
            child("A", "A::b0", "1", 1),
            child("A", "A::b0", "2", 2),
            parent("B", "B::b0"),
            child("B", "B::b0", "1", 3),
            parent("C", "C::b0"),
            child("C", "C::b0", "1", 4),
            child("C", "C::b0", "2", 5),
        ];
        let components_only = vec![2, 4, 6, 7];

        assert_eq!(
            moving_set(&items, &components_only, 3),
            vec![2, 3, 4, 5, 6, 7]
        );
        assert_eq!(moving_set(&items, &components_only, 0), vec![0, 1, 2]);
    }
}
