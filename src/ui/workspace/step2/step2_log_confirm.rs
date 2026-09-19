// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use crate::app::game_authority;
use crate::ui::orchestrator::widgets::dialogs::confirm_dialog::ConfirmDialog;

fn upper_tab(bgee: bool, game_install: &str) -> &'static str {
    if bgee {
        game_authority::first_slot_tab(game_install)
    } else {
        game_authority::TAB_BG2EE
    }
}

#[must_use]
pub fn weidu_log_dialog_text(bgee: bool, game_install: &str) -> (String, String) {
    let tab = upper_tab(bgee, game_install);
    let title = format!("Replace {tab} selections from a WeiDU log?");
    let body = format!(
        "This will overwrite every component selection on the {tab} bucket \
         with the contents of the chosen weidu.log. Make sure the log was \
         produced from the same mod versions you have downloaded — otherwise \
         components may resolve to the wrong rows or fail to install."
    );
    (title, body)
}

#[must_use]
pub const fn weidu_log_confirm<'a>(title: &'a str, body: &'a str) -> ConfirmDialog<'a> {
    ConfirmDialog {
        id_salt: "step2_select_via_weidu_log",
        title,
        body,
        confirm_label: "Pick a weidu.log...",
        cancel_label: "Cancel",
        danger: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_names_the_target_tab() {
        let (t, _) = weidu_log_dialog_text(true, "BGEE");
        assert_eq!(t, "Replace BGEE selections from a WeiDU log?");
        let (t2, _) = weidu_log_dialog_text(false, "BGEE");
        assert_eq!(t2, "Replace BG2EE selections from a WeiDU log?");
    }

    #[test]
    fn body_is_wireframe_verbatim() {
        let (_, b) = weidu_log_dialog_text(true, "BGEE");
        assert_eq!(
            b,
            "This will overwrite every component selection on the BGEE \
             bucket with the contents of the chosen weidu.log. Make sure \
             the log was produced from the same mod versions you have \
             downloaded — otherwise components may resolve to the wrong \
             rows or fail to install."
        );
        let (_, b2) = weidu_log_dialog_text(false, "BGEE");
        assert!(b2.contains("on the BG2EE bucket"));
    }

    #[test]
    fn confirm_descriptor_is_danger_with_wireframe_label() {
        let (t, b) = weidu_log_dialog_text(true, "BGEE");
        let d = weidu_log_confirm(&t, &b);
        assert!(d.danger);
        assert_eq!(d.confirm_label, "Pick a weidu.log...");
        assert_eq!(d.id_salt, "step2_select_via_weidu_log");
    }

    #[test]
    fn log_confirm_text_names_the_lists_own_tab() {
        let (t, _) = weidu_log_dialog_text(true, "IWDEE");
        assert!(t.contains("IWDEE"));
        assert!(!t.contains("BGEE"));
        let (t2, _) = weidu_log_dialog_text(true, "BGEE");
        assert!(t2.contains("BGEE"));
        let (t3, _) = weidu_log_dialog_text(false, "IWDEE");
        assert!(t3.contains("BG2EE"));
    }
}
