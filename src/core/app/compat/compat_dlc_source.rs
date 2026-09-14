// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::btree_map::Entry as BTreeMapEntry;
use std::collections::hash_map::Entry as HashMapEntry;
use std::path::Path;

use crate::app::source_check::{self, SodDlcState};
use crate::app::state::{Step1State, Step2ModState, Step3ItemState};

use super::compat_rule_runtime::{collect_step2_active_items, normalize_mod_key};
use super::compat_step3_rules::{Step3CompatMarker, marker_key};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DlcSourceCheck {
    probes: BTreeMap<String, SodDlcState>,
}

pub(crate) fn refresh_source_check(step1: &mut Step1State) -> bool {
    let wanted = [
        step1.bgee_game_folder.trim(),
        step1.eet_bgee_game_folder.trim(),
    ];
    let probes = &mut step1.dlc_source_check.probes;
    let up_to_date = probes.keys().all(|key| wanted.contains(&key.as_str()))
        && wanted
            .iter()
            .filter(|source| !source.is_empty())
            .all(|source| probes.contains_key(*source));
    if up_to_date {
        return false;
    }
    probes.retain(|key, _| wanted.contains(&key.as_str()));
    for source in wanted.into_iter().filter(|source| !source.is_empty()) {
        if let BTreeMapEntry::Vacant(slot) = probes.entry(source.to_string()) {
            slot.insert(source_check::sod_state(Path::new(source)));
        }
    }
    true
}

#[must_use]
pub(crate) fn bgee_source_for<'a>(step1: &'a Step1State, game: &str) -> &'a str {
    if game == "EET" {
        step1.eet_bgee_game_folder.trim()
    } else {
        step1.bgee_game_folder.trim()
    }
}

fn required(step1: &Step1State) -> bool {
    required_for_game(step1, &step1.game_install)
}

fn required_for_game(step1: &Step1State, game: &str) -> bool {
    matches!(game, "BGEE" | "EET")
        && step1
            .dlc_source_check
            .probes
            .get(bgee_source_for(step1, game))
            .is_some_and(|sod| matches!(sod, SodDlcState::Unmerged { .. }))
}

fn is_merger(tp_file: &str, component: &str) -> bool {
    normalize_mod_key(tp_file) == "dlcmerger" && matches!(component.trim(), "1" | "3")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceNoticeSeverity {
    Info,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceRemedy {
    OrderMerger,
    ChangeSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceNotice {
    pub(crate) severity: SourceNoticeSeverity,
    pub(crate) text: &'static str,
    pub(crate) remedy: SourceRemedy,
}

pub(crate) fn preview_issue(
    step1: &Step1State,
    preview: &crate::app::modlist_share::ModlistSharePreview,
) -> Option<SourceNotice> {
    if !matches!(preview.game_install.as_str(), "BGEE" | "EET") {
        return None;
    }
    let sod = step1
        .dlc_source_check
        .probes
        .get(bgee_source_for(step1, &preview.game_install))?;
    if matches!(sod, SodDlcState::NotApplicable) {
        return None;
    }
    let parse = |text: &str| {
        text.lines()
            .map(str::trim)
            .filter(|line| line.starts_with('~'))
            .filter_map(|line| crate::mods::component::Component::parse_weidu_line(line).ok())
            .collect::<Vec<_>>()
    };
    let bgee = parse(&preview.bgee_log_text);
    let has_tweaks = |items: &[crate::mods::component::Component]| {
        items
            .iter()
            .any(|item| normalize_mod_key(&item.tp_file) == "cdtweaks")
    };
    let merger_position = bgee
        .iter()
        .position(|item| is_merger(&item.tp_file, &item.component));
    if let Some(position) = merger_position {
        return match (position, sod) {
            (0, SodDlcState::Unmerged { .. }) => Some(SourceNotice {
                severity: SourceNoticeSeverity::Info,
                text: "This modlist needs Siege of Dragonspear. Your BGEE source has the DLC archive; DLC Merger merges it during the install.",
                remedy: SourceRemedy::OrderMerger,
            }),
            (_, SodDlcState::Unmerged { .. }) => Some(SourceNotice {
                severity: SourceNoticeSeverity::Warning,
                text: "Move DLC Merger (#1 or #3) first in the BGEE installation order.",
                remedy: SourceRemedy::OrderMerger,
            }),
            (_, SodDlcState::Merged) => Some(SourceNotice {
                severity: SourceNoticeSeverity::Warning,
                text: "This modlist includes DLC Merger, but your BGEE source already has Siege of Dragonspear merged. DLC Merger stops with 'already merged'; remove it before installing.",
                remedy: SourceRemedy::ChangeSource,
            }),
            (_, SodDlcState::Absent) => Some(SourceNotice {
                severity: SourceNoticeSeverity::Warning,
                text: "This modlist needs Siege of Dragonspear, and your BGEE source does not have it. DLC Merger will fail; install from a source that includes the DLC.",
                remedy: SourceRemedy::ChangeSource,
            }),
            (_, SodDlcState::NotApplicable) => None,
        };
    }
    let has_tweaks_anywhere = has_tweaks(&bgee)
        || preview.game_install == "EET" && has_tweaks(&parse(&preview.bg2ee_log_text));
    if has_tweaks_anywhere && matches!(sod, SodDlcState::Unmerged { .. }) {
        return Some(SourceNotice {
            severity: SourceNoticeSeverity::Warning,
            text: "Your BGEE source contains DLC that needs merging. This modlist includes CDTweaks, which requires DLC Merger for this source.",
            remedy: SourceRemedy::OrderMerger,
        });
    }
    None
}

fn marker(step1: &Step1State, kind: &str, component: &str) -> Step3CompatMarker {
    let sod = step1
        .dlc_source_check
        .probes
        .get(bgee_source_for(step1, &step1.game_install));
    Step3CompatMarker {
        kind: kind.to_string(),
        message: Some(if kind == "order_block" {
            "Install DLC Merger (#1 or #3) first in the BGEE installation order.".to_string()
        } else {
            "Requires DLC Merger first in the BGEE installation order: select Merge Siege of Dragonspear (#1) or All available DLCs (#3).".to_string()
        }),
        related_mod: Some("dlcmerger".to_string()),
        related_component: Some(component.to_string()),
        source: Some("BGEE source check".to_string()),
        raw_evidence: match sod {
            Some(SodDlcState::Unmerged { archive }) => Some(format!(
                "Unmerged Siege of Dragonspear archive in BGEE source: {}",
                archive.display()
            )),
            _ => None,
        },
    }
}

pub(crate) fn apply_step2(
    step1: &Step1State,
    first: &mut [Step2ModState],
    second: &mut [Step2ModState],
) {
    if !required(step1) {
        return;
    }
    if collect_step2_active_items(first)
        .iter()
        .any(|item| is_merger(&item.tp_file, &item.component_id))
    {
        return;
    }
    let hit = marker(step1, "missing_dep", "1");
    for mod_state in first.iter_mut().chain(second.iter_mut()) {
        if normalize_mod_key(&mod_state.tp_file) != "cdtweaks" {
            continue;
        }
        for component in &mut mod_state.components {
            if component.compat_kind.is_some() {
                continue;
            }
            component.compat_kind = Some(hit.kind.clone());
            component.compat_source.clone_from(&hit.source);
            component.compat_related_mod.clone_from(&hit.related_mod);
            component
                .compat_related_component
                .clone_from(&hit.related_component);
            component.compat_evidence.clone_from(&hit.raw_evidence);
            component.disabled_reason.clone_from(&hit.message);
        }
    }
}

pub(crate) fn apply_step3(
    step1: &Step1State,
    _tab: &str,
    items: &[Step3ItemState],
    bgee_items: &[Step3ItemState],
    markers: &mut HashMap<String, Step3CompatMarker>,
) {
    if !required(step1) {
        return;
    }
    let merger = bgee_items
        .iter()
        .filter(|item| !item.is_parent)
        .enumerate()
        .find(|(_, item)| is_merger(&item.tp_file, &item.component_id));
    for item in items.iter().filter(|item| !item.is_parent) {
        if normalize_mod_key(&item.tp_file) != "cdtweaks" {
            continue;
        }
        let HashMapEntry::Vacant(slot) = markers.entry(marker_key(item)) else {
            continue;
        };
        let hit = match merger {
            None => marker(step1, "missing_dep", "1"),
            Some((order, dependency)) if order != 0 => {
                marker(step1, "order_block", &dependency.component_id)
            }
            Some(_) => continue,
        };
        slot.insert(hit);
    }
}

#[cfg(test)]
#[path = "compat_dlc_source_tests.rs"]
mod tests;
