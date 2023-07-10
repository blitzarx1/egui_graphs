use egui::{ScrollArea, Ui};

use super::style::header_accent;

const SIZE_SPACE: f32 = 10.;

pub fn draw_view_toolbox(ui: &mut Ui, loading: bool) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(SIZE_SPACE);
            ui.label(header_accent("toolbox"));
            ui.add_space(SIZE_SPACE);
            ui.separator();

            ui.add_space(SIZE_SPACE);
            ui.label(match loading {
                true => "scrapping links",
                false => "",
            });

            if loading {
                ui.centered_and_justified(|ui| ui.spinner());
                return;
            }
        });
    });
}
