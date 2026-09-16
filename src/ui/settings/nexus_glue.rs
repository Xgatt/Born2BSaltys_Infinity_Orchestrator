// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use eframe::egui;

use crate::app::nexus_auth::{
    clear_nexus_api_key, poll_nexus_key_validation, start_nexus_key_validation,
};
use crate::ui::orchestrator::orchestrator_app::OrchestratorApp;
use crate::ui::settings::nexus_connect_dialog::{self, NexusConnectDialog, NexusConnectOutcome};

const CHECKING_STATUS_TEXT: &str = "Checking your key with Nexus Mods…";

pub fn open_nexus_dialog(orchestrator: &mut OrchestratorApp) {
    let checking = orchestrator.nexus_auth_rx.is_some();
    let dialog = &mut orchestrator.settings_screen_state.nexus_dialog;
    if dialog.open {
        return;
    }
    dialog.open = true;
    dialog.focus_pending = true;
    dialog.validating = checking;
    dialog.key_input.clear();
    dialog.status_text = if checking {
        CHECKING_STATUS_TEXT.to_string()
    } else {
        String::new()
    };
}

pub fn submit_nexus_key(orchestrator: &mut OrchestratorApp) {
    if orchestrator.nexus_auth_rx.is_some() {
        return;
    }
    let dialog = &mut orchestrator.settings_screen_state.nexus_dialog;
    if dialog.key_input.trim().is_empty() {
        return;
    }
    dialog.validating = true;
    dialog.status_text = CHECKING_STATUS_TEXT.to_string();
    let key = dialog.key_input.trim().to_string();
    orchestrator.nexus_auth_rx = Some(start_nexus_key_validation(key));
}

pub fn cancel_nexus_dialog(orchestrator: &mut OrchestratorApp) {
    let dialog = &mut orchestrator.settings_screen_state.nexus_dialog;
    dialog.open = false;
    dialog.key_input.clear();
    dialog.status_text.clear();
    dialog.validating = false;
}

pub fn poll_nexus_validation(orchestrator: &mut OrchestratorApp) {
    let Some(result) = poll_nexus_key_validation(&mut orchestrator.nexus_auth_rx) else {
        return;
    };
    match result {
        Ok(account) => {
            let dialog = &mut orchestrator.settings_screen_state.nexus_dialog;
            if dialog.open {
                dialog.open = false;
                dialog.key_input.clear();
                dialog.status_text.clear();
            }
            dialog.validating = false;
            orchestrator
                .notification_manager
                .success(format!("Connected to Nexus Mods as {}", account.name));
            orchestrator.nexus_account = Some(account);
        }
        Err(text) => {
            let dialog = &mut orchestrator.settings_screen_state.nexus_dialog;
            dialog.validating = false;
            if dialog.open {
                dialog.status_text = text;
            }
        }
    }
}

pub fn disconnect_nexus(orchestrator: &mut OrchestratorApp) {
    disconnect_nexus_state_only(orchestrator);
    match clear_nexus_api_key() {
        Ok(()) => orchestrator
            .notification_manager
            .success("Nexus Mods disconnected."),
        Err(err) => orchestrator
            .notification_manager
            .error(format!("Nexus Mods disconnect failed: {err}")),
    }
}

pub(crate) fn disconnect_nexus_state_only(orchestrator: &mut OrchestratorApp) {
    orchestrator.nexus_auth_rx = None;
    orchestrator.nexus_account = None;
    let dialog = &mut orchestrator.settings_screen_state.nexus_dialog;
    dialog.open = false;
    dialog.key_input.clear();
    dialog.status_text.clear();
    dialog.validating = false;
}

pub fn render_nexus_dialog_if_open(orchestrator: &mut OrchestratorApp, ctx: &egui::Context) {
    if !orchestrator.settings_screen_state.nexus_dialog.open {
        return;
    }
    let palette = orchestrator.theme_palette;
    let outcome = {
        let dialog = &mut orchestrator.settings_screen_state.nexus_dialog;
        let status_text = dialog.status_text.clone();
        nexus_connect_dialog::render(
            ctx,
            palette,
            &mut NexusConnectDialog {
                key_input: &mut dialog.key_input,
                focus_pending: &mut dialog.focus_pending,
                status_text: &status_text,
                validating: dialog.validating,
            },
        )
    };
    match outcome {
        NexusConnectOutcome::Submit => submit_nexus_key(orchestrator),
        NexusConnectOutcome::Cancel => cancel_nexus_dialog(orchestrator),
        NexusConnectOutcome::Pending => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::nexus_auth::NexusAccount;
    use std::sync::mpsc;

    fn account(name: &str, is_premium: bool) -> NexusAccount {
        NexusAccount {
            name: name.to_string(),
            is_premium,
        }
    }

    #[test]
    fn a_successful_result_closes_the_dialog_sets_the_account_and_clears_the_channel() {
        let mut app = OrchestratorApp::new_isolated_for_test("nexusglue");
        let (tx, rx) = mpsc::channel();
        app.nexus_auth_rx = Some(rx);
        app.settings_screen_state.nexus_dialog.open = true;
        app.settings_screen_state.nexus_dialog.validating = true;
        tx.send(Ok(account("Xgatt", true))).unwrap();

        poll_nexus_validation(&mut app);

        assert!(!app.settings_screen_state.nexus_dialog.open);
        assert!(app.settings_screen_state.nexus_dialog.key_input.is_empty());
        assert!(
            app.settings_screen_state
                .nexus_dialog
                .status_text
                .is_empty()
        );
        assert!(!app.settings_screen_state.nexus_dialog.validating);
        assert_eq!(app.nexus_account, Some(account("Xgatt", true)));
        assert!(app.nexus_auth_rx.is_none());
    }

    #[test]
    fn a_failed_result_keeps_the_dialog_open_with_the_message() {
        let mut app = OrchestratorApp::new_isolated_for_test("nexusglue");
        let (tx, rx) = mpsc::channel();
        app.nexus_auth_rx = Some(rx);
        app.settings_screen_state.nexus_dialog.open = true;
        app.settings_screen_state.nexus_dialog.validating = true;
        tx.send(Err("nope".to_string())).unwrap();

        poll_nexus_validation(&mut app);

        assert!(app.settings_screen_state.nexus_dialog.open);
        assert_eq!(app.settings_screen_state.nexus_dialog.status_text, "nope");
        assert!(!app.settings_screen_state.nexus_dialog.validating);
        assert!(app.nexus_account.is_none());
    }

    #[test]
    fn a_result_after_cancel_still_sets_the_account_but_writes_no_status() {
        let mut app = OrchestratorApp::new_isolated_for_test("nexusglue");
        let (tx, rx) = mpsc::channel();
        app.nexus_auth_rx = Some(rx);
        app.settings_screen_state.nexus_dialog.open = true;
        cancel_nexus_dialog(&mut app);
        tx.send(Ok(account("Xgatt", false))).unwrap();

        poll_nexus_validation(&mut app);

        assert!(!app.settings_screen_state.nexus_dialog.open);
        assert!(
            app.settings_screen_state
                .nexus_dialog
                .status_text
                .is_empty()
        );
        assert_eq!(app.nexus_account, Some(account("Xgatt", false)));
    }

    #[test]
    fn submit_ignores_a_blank_key() {
        let mut app = OrchestratorApp::new_isolated_for_test("nexusglue");
        app.settings_screen_state.nexus_dialog.key_input = "   ".to_string();

        submit_nexus_key(&mut app);

        assert!(app.nexus_auth_rx.is_none());
        assert!(!app.settings_screen_state.nexus_dialog.validating);
    }

    #[test]
    fn open_while_a_check_runs_shows_the_checking_status_and_asks_for_focus() {
        let mut app = OrchestratorApp::new_isolated_for_test("nexusglue");
        let (_tx, rx) = mpsc::channel();
        app.nexus_auth_rx = Some(rx);

        open_nexus_dialog(&mut app);

        let dialog = &app.settings_screen_state.nexus_dialog;
        assert!(dialog.open);
        assert!(dialog.focus_pending);
        assert!(dialog.validating);
        assert_eq!(dialog.status_text, CHECKING_STATUS_TEXT);
    }

    #[test]
    fn open_while_already_open_keeps_the_typed_key() {
        let mut app = OrchestratorApp::new_isolated_for_test("nexusglue");
        open_nexus_dialog(&mut app);
        app.settings_screen_state.nexus_dialog.focus_pending = false;
        app.settings_screen_state.nexus_dialog.key_input = "typed".to_string();

        open_nexus_dialog(&mut app);

        let dialog = &app.settings_screen_state.nexus_dialog;
        assert_eq!(dialog.key_input, "typed");
        assert!(!dialog.focus_pending);
    }

    #[test]
    fn disconnect_clears_the_account_and_the_channel() {
        let mut app = OrchestratorApp::new_isolated_for_test("nexusglue");
        let (_tx, rx) = mpsc::channel();
        app.nexus_auth_rx = Some(rx);
        app.nexus_account = Some(account("Xgatt", true));
        app.settings_screen_state.nexus_dialog.open = true;

        disconnect_nexus_state_only(&mut app);

        assert!(app.nexus_auth_rx.is_none());
        assert!(app.nexus_account.is_none());
        assert!(!app.settings_screen_state.nexus_dialog.open);
    }
}
