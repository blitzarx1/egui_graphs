use egui::{ScrollArea, Ui};

use super::style::header_accent;

const HEADING: &str = "Wiki Links";
const MSG_SCRAPPING: &str = "scrapping links";

pub struct State {
    pub loading: bool,
    pub spacing: f32,
}

pub fn draw_view_toolbox(ui: &mut Ui, state: &mut State) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(state.spacing);
            ui.label(header_accent(HEADING));
            ui.add_space(state.spacing);
            ui.separator();

            ui.add_space(state.spacing);
            ui.label(match state.loading {
                true => MSG_SCRAPPING,
                false => "",
            });

            if state.loading {
                ui.centered_and_justified(|ui| ui.spinner());
                return;
            }
        });
    });
}
