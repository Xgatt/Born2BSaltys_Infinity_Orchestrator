// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use eframe::egui;

use crate::ui::orchestrator::orchestrator_app::OrchestratorApp;
use crate::ui::orchestrator::widgets::{BtnOpts, InputOpts, redesign_btn, redesign_text_input};
use crate::ui::shared::redesign_tokens::{
    REDESIGN_BORDER_RADIUS_U8, REDESIGN_BORDER_WIDTH_PX, ThemePalette, redesign_border_strong,
    redesign_input_bg, redesign_shell_bg, redesign_text_primary,
};
use crate::ui::step2::action_step2::Step2Action;
use crate::ui::workspace::step2::step2_rescan_reconcile;

const SEARCH_INPUT_H: f32 = 30.0;
const ROW_GAP: f32 = 10.0;
const SEARCH_INPUT_TEXT_PAD: i8 = 8;
const DROPDOWN_MIN_W: f32 = 160.0;
const TRIGGER_LABEL: &str = "Rescan Mods";
const TRIGGER_PAD_X: f32 = 10.0;
const TRIGGER_PAD_Y: f32 = 4.0;
const TRIGGER_FONT_SIZE: f32 = 12.0;
const CARET_GAP: f32 = 7.0;
const CARET_W: f32 = 9.0;
const CARET_H: f32 = 5.0;

const RESCAN_DISABLED_TIP: &str = "Available after install prep (Phase 7) \u{2014} \
     the mods folder is extracted per-install at prep time (SPEC \u{00A7}13.12a). \
     Use \u{201C}Global mods folder\u{201D} to scan your configured mods folder in the meantime.";

const GLOBAL_MODS_DISABLED_TIP: &str =
    "No mods folder configured. Set it in Settings > Paths > Mods folder.";

pub fn render(
    ui: &mut egui::Ui,
    orchestrator: &mut OrchestratorApp,
    palette: ThemePalette,
    rect: egui::Rect,
) -> Option<Step2Action> {
    let is_scanning = orchestrator.wizard_state.step2.is_scanning;
    let scratch_scan_enabled = scratch_scan_enabled(orchestrator);
    let global_mods_folder = orchestrator
        .settings_store
        .load()
        .ok()
        .map(|s| s.step1.mods_folder)
        .unwrap_or_default();
    let global_enabled = !global_mods_folder.trim().is_empty();

    let mut action: Option<Step2Action> = None;

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = ROW_GAP;

            let trigger_w = if is_scanning {
                small_btn_width(ui, "Cancel Scan")
            } else {
                rescan_trigger_width(ui)
            };
            let search_w = (rect.width() - trigger_w - ROW_GAP).max(80.0);

            let search_margin = egui::Margin::symmetric(SEARCH_INPUT_TEXT_PAD, 4);
            let _resp = redesign_text_input(
                ui,
                palette,
                InputOpts {
                    edit: egui::TextEdit::singleline(
                        &mut orchestrator.wizard_state.step2.search_query,
                    )
                    .hint_text("Search mods or components...")
                    .text_color(redesign_text_primary(palette))
                    .background_color(redesign_input_bg(palette))
                    .margin(search_margin)
                    .font(egui::FontId::new(
                        14.0,
                        egui::FontFamily::Name("poppins_medium".into()),
                    )),
                    margin: search_margin,
                    size: egui::vec2(search_w, SEARCH_INPUT_H),
                    border: None,
                },
            );

            if is_scanning {
                if redesign_btn(
                    ui,
                    palette,
                    "Cancel Scan",
                    BtnOpts {
                        small: true,
                        ..Default::default()
                    },
                )
                .on_hover_text("Stop the running scan and return to idle.")
                .clicked()
                {
                    action = Some(Step2Action::CancelScan);
                }
            } else {
                let trigger = rescan_trigger(ui, palette);
                let popup_id = ui.make_persistent_id("step2_rescan_mods_dropdown");
                if trigger.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
                let chosen = rescan_dropdown(
                    ui,
                    palette,
                    popup_id,
                    &trigger,
                    scratch_scan_enabled,
                    global_enabled,
                );
                match chosen {
                    DropdownChoice::InstallationFolder => {
                        let scratch = orchestrator
                            .workspace_state
                            .get(orchestrator.workspace_view.modlist_id.trim())
                            .and_then(|w| w.scratch_mods_folder.clone())
                            .unwrap_or_default();
                        orchestrator.wizard_state.step1.mods_folder = scratch;
                        step2_rescan_reconcile::snapshot_current_selection(orchestrator);
                        action = Some(Step2Action::StartScan);
                    }
                    DropdownChoice::GlobalModsFolder => {
                        orchestrator.workspace_view.step2.pending_global_mods_scan = Some(());
                    }
                    DropdownChoice::None => {}
                }
            }
        });
    });

    action
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DropdownChoice {
    None,
    InstallationFolder,
    GlobalModsFolder,
}

fn rescan_dropdown(
    ui: &egui::Ui,
    palette: ThemePalette,
    popup_id: egui::Id,
    trigger: &egui::Response,
    scratch_enabled: bool,
    global_enabled: bool,
) -> DropdownChoice {
    if !ui.memory(|mem| mem.is_popup_open(popup_id)) {
        return DropdownChoice::None;
    }

    let mut pos = trigger.rect.right_bottom();
    if let Some(to_global) = ui.ctx().layer_transform_to_global(ui.layer_id()) {
        pos = to_global * pos;
    }

    let frame = egui::Frame::popup(ui.style());
    let area_resp = egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .pivot(egui::Align2::RIGHT_TOP)
        .show(ui.ctx(), |inner_ui| {
            frame
                .show(inner_ui, |inner_ui| {
                    inner_ui.set_max_width(DROPDOWN_MIN_W);

                    let chassis = egui::Frame::default()
                        .fill(redesign_shell_bg(palette))
                        .stroke(egui::Stroke::new(
                            REDESIGN_BORDER_WIDTH_PX,
                            redesign_border_strong(palette),
                        ))
                        .corner_radius(egui::CornerRadius::same(REDESIGN_BORDER_RADIUS_U8))
                        .inner_margin(egui::Margin::same(4));

                    chassis
                        .show(inner_ui, |inner_ui| {
                            inner_ui.spacing_mut().item_spacing.y = 0.0;
                            let mut choice = DropdownChoice::None;

                            let inst_clicked = dropdown_item(
                                inner_ui,
                                palette,
                                "Installation folder",
                                scratch_enabled,
                                if scratch_enabled {
                                    None
                                } else {
                                    Some(RESCAN_DISABLED_TIP)
                                },
                            );
                            if inst_clicked {
                                choice = DropdownChoice::InstallationFolder;
                            }

                            let global_clicked = dropdown_item(
                                inner_ui,
                                palette,
                                "Global mods folder",
                                global_enabled,
                                if global_enabled {
                                    None
                                } else {
                                    Some(GLOBAL_MODS_DISABLED_TIP)
                                },
                            );
                            if global_clicked {
                                choice = DropdownChoice::GlobalModsFolder;
                            }

                            choice
                        })
                        .inner
                })
                .inner
        });

    let should_close = trigger.clicked_elsewhere() && area_resp.response.clicked_elsewhere();
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) || should_close {
        ui.memory_mut(egui::Memory::close_popup);
    }

    let choice = area_resp.inner;
    if choice != DropdownChoice::None {
        ui.memory_mut(egui::Memory::close_popup);
    }
    choice
}

fn dropdown_item(
    ui: &mut egui::Ui,
    palette: ThemePalette,
    label: &str,
    enabled: bool,
    tooltip: Option<&str>,
) -> bool {
    use crate::ui::shared::redesign_tokens::{redesign_hover_overlay, redesign_text_faint};

    let text_color = if enabled {
        redesign_text_primary(palette)
    } else {
        redesign_text_faint(palette)
    };
    let font = egui::FontId::new(13.0, egui::FontFamily::Name("poppins_medium".into()));

    let pad_x = 10.0;
    let pad_y = 6.0;
    let row_width = ui.available_width().max(DROPDOWN_MIN_W - 8.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color);
    let row_height = galley.size().y + pad_y * 2.0;
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(row_width, row_height), sense);

    if ui.is_rect_visible(rect) {
        if response.hovered() && enabled {
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same(REDESIGN_BORDER_RADIUS_U8),
                redesign_hover_overlay(palette),
            );
        }
        ui.painter().text(
            egui::pos2(rect.left() + pad_x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            font,
            text_color,
        );
    }

    if let Some(tip) = tooltip {
        response.on_hover_text(tip).clicked()
    } else {
        response.clicked()
    }
}

fn scratch_scan_enabled(orchestrator: &OrchestratorApp) -> bool {
    let id = orchestrator.workspace_view.modlist_id.trim();
    if id.is_empty()
        || orchestrator
            .wizard_state
            .step1
            .mods_folder
            .trim()
            .is_empty()
    {
        return false;
    }
    orchestrator
        .workspace_state
        .get(id)
        .and_then(|workspace| workspace.scratch_mods_folder.as_deref())
        .is_some_and(|folder| !folder.trim().is_empty())
}

fn small_btn_width(ui: &egui::Ui, label: &str) -> f32 {
    let font = egui::FontId::new(12.0, egui::FontFamily::Name("poppins_medium".into()));
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font, egui::Color32::WHITE);
    10.0_f32.mul_add(2.0, galley.size().x)
}

fn trigger_font() -> egui::FontId {
    egui::FontId::new(
        TRIGGER_FONT_SIZE,
        egui::FontFamily::Name("poppins_medium".into()),
    )
}

fn rescan_trigger_width(ui: &egui::Ui) -> f32 {
    let galley = ui.painter().layout_no_wrap(
        TRIGGER_LABEL.to_string(),
        trigger_font(),
        egui::Color32::WHITE,
    );
    TRIGGER_PAD_X.mul_add(2.0, galley.size().x) + CARET_GAP + CARET_W
}

fn rescan_trigger(ui: &mut egui::Ui, palette: ThemePalette) -> egui::Response {
    let text_color = redesign_text_primary(palette);
    let font = trigger_font();
    let galley = ui
        .painter()
        .layout_no_wrap(TRIGGER_LABEL.to_string(), font.clone(), text_color);
    let content_w = galley.size().x + CARET_GAP + CARET_W;
    let size = egui::vec2(
        TRIGGER_PAD_X.mul_add(2.0, content_w),
        TRIGGER_PAD_Y.mul_add(2.0, galley.size().y),
    );
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let rect = if response.is_pointer_button_down_on() {
        rect.translate(egui::vec2(1.0, 1.0))
    } else {
        rect
    };

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = egui::CornerRadius::same(REDESIGN_BORDER_RADIUS_U8);
        painter.rect_filled(rect, radius, redesign_shell_bg(palette));
        painter.rect_stroke(
            rect,
            radius,
            egui::Stroke::new(REDESIGN_BORDER_WIDTH_PX, redesign_border_strong(palette)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            egui::pos2(rect.left() + TRIGGER_PAD_X, rect.center().y),
            egui::Align2::LEFT_CENTER,
            TRIGGER_LABEL,
            font,
            text_color,
        );
        let caret_cx = rect.left() + TRIGGER_PAD_X + galley.size().x + CARET_GAP + CARET_W / 2.0;
        paint_down_caret(painter, egui::pos2(caret_cx, rect.center().y), text_color);
    }

    response
}

fn paint_down_caret(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let hw = CARET_W / 2.0;
    let hh = CARET_H / 2.0;
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(center.x - hw, center.y - hh),
            egui::pos2(center.x + hw, center.y - hh),
            egui::pos2(center.x, center.y + hh),
        ],
        color,
        egui::Stroke::NONE,
    ));
}
