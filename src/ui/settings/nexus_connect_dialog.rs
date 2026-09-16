// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use eframe::egui;

use crate::app::controller::util::open_in_shell;
use crate::ui::orchestrator::widgets::{BtnOpts, InputOpts, redesign_btn, redesign_text_input};
use crate::ui::shared::redesign_tokens::{
    REDESIGN_BORDER_RADIUS_U8, REDESIGN_BORDER_WIDTH_PX, ThemePalette, redesign_border_strong,
    redesign_input_bg, redesign_pill_danger, redesign_shell_bg, redesign_text_muted,
    redesign_text_primary,
};

pub struct NexusConnectDialog<'a> {
    pub key_input: &'a mut String,
    pub focus_pending: &'a mut bool,
    pub status_text: &'a str,
    pub validating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NexusConnectOutcome {
    #[default]
    Pending,
    Submit,
    Cancel,
}

struct KeyFieldEvents {
    enter_pressed: bool,
}

const MAX_WIDTH_PX: f32 = 460.0;
const GUIDANCE_TEXT: &str = "Paste the personal API key from your Nexus Mods account. BIO keeps it in this computer's credential store and never writes it into a modlist or a share code.";
const API_KEYS_PAGE_URL: &str = "https://next.nexusmods.com/settings/api-keys";

#[must_use]
pub fn render(
    ctx: &egui::Context,
    palette: ThemePalette,
    dialog: &mut NexusConnectDialog<'_>,
) -> NexusConnectOutcome {
    let mut outcome = NexusConnectOutcome::Pending;

    let dialog_id = egui::Id::new("orchestrator_nexus_connect_dialog");

    let frame = egui::Frame::default()
        .fill(redesign_shell_bg(palette))
        .stroke(egui::Stroke::new(
            REDESIGN_BORDER_WIDTH_PX,
            redesign_border_strong(palette),
        ))
        .corner_radius(egui::CornerRadius::same(REDESIGN_BORDER_RADIUS_U8))
        .inner_margin(egui::Margin::same(18));

    egui::Window::new("Connect Nexus Mods")
        .id(dialog_id)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .frame(frame)
        .show(ctx, |ui| {
            ui.set_max_width(MAX_WIDTH_PX);

            ui.label(
                egui::RichText::new("Connect Nexus Mods")
                    .size(15.0)
                    .family(egui::FontFamily::Name("poppins_medium".into()))
                    .color(redesign_text_primary(palette)),
            );
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(GUIDANCE_TEXT)
                    .size(13.0)
                    .family(egui::FontFamily::Name("poppins_light".into()))
                    .color(redesign_text_muted(palette)),
            );
            ui.add_space(8.0);

            if *dialog.focus_pending {
                ui.memory_mut(|m| m.data.remove::<String>(dialog_id.with("open_error")));
            }
            render_open_api_keys_page_button(ui, palette, dialog_id);

            ui.add_space(12.0);

            let key_events = render_key_field(
                ui,
                palette,
                dialog_id,
                dialog.key_input,
                dialog.focus_pending,
            );

            if !dialog.status_text.is_empty() {
                ui.add_space(6.0);
                let status_color = if dialog.validating {
                    redesign_text_muted(palette)
                } else {
                    redesign_pill_danger(palette)
                };
                ui.label(
                    egui::RichText::new(dialog.status_text)
                        .size(12.0)
                        .color(status_color),
                );
            }

            ui.add_space(14.0);

            let connect_disabled = dialog.validating || dialog.key_input.trim().is_empty();

            if let Some(footer_outcome) =
                render_footer(ui, palette, connect_disabled, key_events.enter_pressed)
            {
                outcome = footer_outcome;
            }
        });

    outcome
}

fn render_open_api_keys_page_button(ui: &mut egui::Ui, palette: ThemePalette, dialog_id: egui::Id) {
    let clicked = redesign_btn(
        ui,
        palette,
        "Open the API keys page",
        BtnOpts {
            small: true,
            ..Default::default()
        },
    )
    .clicked();

    let error_marker = dialog_id.with("open_error");
    if clicked {
        match open_in_shell(API_KEYS_PAGE_URL) {
            Ok(()) => ui.memory_mut(|m| m.data.remove::<String>(error_marker)),
            Err(err) => ui.memory_mut(|m| {
                m.data
                    .insert_temp(error_marker, format!("Could not open the browser: {err}"));
            }),
        }
    }

    let open_error = ui.memory(|m| m.data.get_temp::<String>(error_marker));
    if let Some(open_error) = open_error {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(open_error)
                .size(12.0)
                .color(redesign_pill_danger(palette)),
        );
    }
}

fn render_key_field(
    ui: &mut egui::Ui,
    palette: ThemePalette,
    dialog_id: egui::Id,
    key_input: &mut String,
    focus_pending: &mut bool,
) -> KeyFieldEvents {
    ui.label(
        egui::RichText::new("Personal API key")
            .size(12.0)
            .family(egui::FontFamily::Name("poppins_medium".into()))
            .color(redesign_text_muted(palette)),
    );
    ui.add_space(4.0);

    let key_id = dialog_id.with("key_field");
    let key_response = redesign_text_input(
        ui,
        palette,
        InputOpts {
            edit: egui::TextEdit::singleline(key_input)
                .id(key_id)
                .password(true)
                .hint_text("paste your key")
                .font(egui::FontId::new(
                    14.0,
                    egui::FontFamily::Name("poppins_light".into()),
                ))
                .text_color(redesign_text_primary(palette))
                .background_color(redesign_input_bg(palette))
                .margin(egui::Margin::symmetric(8, 4)),
            margin: egui::Margin::symmetric(8, 4),
            size: egui::vec2(ui.available_width(), 30.0),
            border: None,
        },
    );

    if *focus_pending {
        key_response.request_focus();
        *focus_pending = false;
    }

    let enter_pressed = (key_response.has_focus() || key_response.lost_focus())
        && ui.input(|i| i.key_pressed(egui::Key::Enter));

    KeyFieldEvents { enter_pressed }
}

fn render_footer(
    ui: &mut egui::Ui,
    palette: ThemePalette,
    connect_disabled: bool,
    enter_pressed: bool,
) -> Option<NexusConnectOutcome> {
    let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

    let mut outcome = None;
    let footer_h = 30.0;
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), footer_h),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            let connect_clicked = redesign_btn(
                ui,
                palette,
                "Connect",
                BtnOpts {
                    small: true,
                    primary: true,
                    disabled: connect_disabled,
                    ..Default::default()
                },
            )
            .clicked();

            let cancel_clicked = redesign_btn(
                ui,
                palette,
                "Cancel",
                BtnOpts {
                    small: true,
                    ..Default::default()
                },
            )
            .clicked();

            if connect_clicked || (enter_pressed && !connect_disabled) {
                outcome = Some(NexusConnectOutcome::Submit);
            } else if cancel_clicked || escape_pressed {
                outcome = Some(NexusConnectOutcome::Cancel);
            }
        },
    );

    outcome
}
