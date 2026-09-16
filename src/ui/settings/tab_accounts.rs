// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use eframe::egui;

use crate::ui::orchestrator::orchestrator_app::OrchestratorApp;
use crate::ui::settings::widgets::account_card::{self, AccountCard, CardState};
use crate::ui::settings::{nexus_glue, oauth_glue};

pub fn render(ui: &mut egui::Ui, orchestrator: &mut OrchestratorApp) {
    let palette = orchestrator.theme_palette;
    let login = orchestrator.wizard_state.github_auth_login.clone();

    let gh_state = if login.trim().is_empty() {
        CardState::NotConnected
    } else {
        CardState::Connected {
            user_label: login.trim(),
            badge: None,
        }
    };
    let gh_clicked = account_card::render(
        ui,
        palette,
        AccountCard {
            initials: "GH",
            service_name: "GitHub",
            state: gh_state,
            connect_label: "connect",
            disconnect_label: "disconnect",
            disabled: false,
        },
    );
    if gh_clicked {
        if login.trim().is_empty() {
            oauth_glue::start_github_flow(orchestrator, false);
        } else {
            oauth_glue::disconnect_github(orchestrator);
        }
    }

    let nexus = orchestrator.nexus_account.clone();
    let nx_state = nexus
        .as_ref()
        .map_or(CardState::NotConnected, |account| CardState::Connected {
            user_label: account.name.as_str(),
            badge: Some(if account.is_premium {
                "premium"
            } else {
                "free"
            }),
        });
    let nx_clicked = account_card::render(
        ui,
        palette,
        AccountCard {
            initials: "NX",
            service_name: "Nexus Mods",
            state: nx_state,
            connect_label: "connect",
            disconnect_label: "disconnect",
            disabled: false,
        },
    );
    if nx_clicked {
        if nexus.is_some() {
            nexus_glue::disconnect_nexus(orchestrator);
        } else {
            nexus_glue::open_nexus_dialog(orchestrator);
        }
    }

    let _ = account_card::render(
        ui,
        palette,
        AccountCard {
            initials: "M",
            service_name: "Mega",
            state: CardState::NotConnected,
            connect_label: "connect",
            disconnect_label: "disconnect",
            disabled: true,
        },
    );
}
