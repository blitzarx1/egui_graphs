use egui::{Align, Response, TextEdit, Ui};

use super::style::{header_accent, COLOR_ERROR, FONT_INPUT};

const HEADING: &str = "Wiki Links";
const DESCRIPTION : &str = "A demo application for egui_graphs widget. This application will display a graph of a wikipedia article links.";
const TOOLTIP: &str = "enter a Wikipedia article url and hit enter";
const ERROR_MSG: &str = "enter a valid wikipedia article url";

pub fn draw_view_input(
    root_article_url: &mut String,
    ui: &mut Ui,
    url_valid: bool,
    size_section: f32,
    size_margin: f32,
) -> Response {
    ui.vertical_centered(|ui| {
        ui.add_space(size_section);
        ui.label(header_accent(HEADING));

        ui.add_space(size_margin);
        ui.label(DESCRIPTION);

        ui.add_space(size_section);
        ui.label(TOOLTIP);

        ui.add_space(size_margin);
        let mut input = TextEdit::singleline(root_article_url)
            .frame(false)
            .desired_rows(1)
            .vertical_align(Align::Center)
            .font(FONT_INPUT)
            .horizontal_align(Align::Center)
            .desired_width(f32::INFINITY);

        if !url_valid {
            input = input.text_color(COLOR_ERROR);
        }

        let input_response = input.show(ui).response;
        input_response.request_focus();

        if !url_valid {
            ui.add_space(size_margin / 4.);
            ui.label(ERROR_MSG);
        };

        input_response
    })
    .inner
}
