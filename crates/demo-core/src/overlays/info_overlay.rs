use crate::ui_consts::{INFO_TEXT_SIZE, UI_MARGIN};
use egui::{self, Align, Layout, RichText};

pub fn render_info_overlay(ui: &mut egui::Ui) {
    // Derive version string: prefer Git tag/describe, then short hash, else crate version
    let cargo_ver = env!("CARGO_PKG_VERSION");
    let version_text = if let Some(desc) = option_env!("DEMO_GIT_DESCRIBE") {
        desc.to_string()
    } else if let Some(hash) = option_env!("DEMO_GIT_HASH") {
        hash.to_string()
    } else {
        cargo_ver.to_string()
    };

    // Prefer override via env, else use CARGO_PKG_REPOSITORY, else default to GitHub
    let repo_url = option_env!("DEMO_REPOSITORY_URL")
        .or(option_env!("CARGO_PKG_REPOSITORY"))
        .unwrap_or("https://github.com/blitzarx1/egui_graphs");

    let panel_rect = ui.max_rect();
    // Match the vertical level of the bottom-right buttons: bottom - UI_MARGIN - btn_height

    // Pre-measure to compute centered position
    let label_galley = ui.fonts_mut(|f| {
        f.layout_no_wrap(
            version_text.clone(),
            egui::FontId::proportional(INFO_TEXT_SIZE),
            ui.style().visuals.text_color(),
        )
    });
    let link_galley = ui.fonts_mut(|f| {
        f.layout_no_wrap(
            "code".into(),
            egui::FontId::proportional(INFO_TEXT_SIZE),
            ui.visuals().hyperlink_color,
        )
    });
    let text_h = label_galley.size().y.max(link_galley.size().y);
    let link_w = link_galley.size().x;
    let y_top = panel_rect.bottom() - UI_MARGIN - text_h;
    let left_pos = egui::pos2(panel_rect.left() + UI_MARGIN, y_top);
    let center_pos = egui::pos2(panel_rect.center().x - link_w * 0.5, y_top);

    // Left: version text
    egui::Area::new(egui::Id::new("demo_info_overlay_version"))
        .order(egui::Order::Middle)
        .fixed_pos(left_pos)
        .movable(false)
        .show(ui.ctx(), |ui_area| {
            ui_area.set_clip_rect(panel_rect);
            ui_area.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                ui.label(RichText::new(version_text.clone()).size(11.0));
            });
        });

    // Center: source code link
    egui::Area::new(egui::Id::new("demo_info_overlay_link"))
        .order(egui::Order::Middle)
        .fixed_pos(center_pos)
        .movable(false)
        .show(ui.ctx(), |ui_area| {
            ui_area.set_clip_rect(panel_rect);
            ui_area.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                let link_text = RichText::new("code")
                    .underline()
                    .size(11.0)
                    .color(ui.visuals().hyperlink_color);
                ui.hyperlink_to(link_text, repo_url);
            });
        });
}
