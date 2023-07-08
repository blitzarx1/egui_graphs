use egui::{
    text::LayoutJob, Align, Button, Context, CursorIcon, FontFamily, FontId, InputState, Label,
    Sense, Stroke, Style, TextEdit, TextFormat, TextStyle, WidgetText,
};

use crate::state::State;

const HEADING: &str = "Wiki Links";
const DESCRIPTION : &str = "A demo application for egui_graphs widget. This application will display a graph of a wikipedia article links.";
const TOOLTIP: &str = "enter a Wikipedia article url and hit enter";

const COLOR_ACCENT: egui::Color32 = egui::Color32::from_rgb(128, 128, 255);
const CURSOR_WIDTH: f32 = 5.;

#[derive(Default)]
pub struct App {
    root_article_url: String,
    state: State,

    size_section: f32,
    size_margin: f32,
    style: Style,
}

impl App {
    pub fn new() -> Self {
        let mut style = Style::default();
        style.visuals.text_cursor_width = CURSOR_WIDTH;
        style.visuals.selection.stroke = Stroke::new(1., COLOR_ACCENT);

        App {
            style,
            ..Default::default()
        }
    }

    pub fn run(&mut self, ctx: &Context, ui: &mut egui::Ui) {
        self.size_section = ui.available_height() / 5.;
        self.size_margin = ui.available_height() / 20.;

        ui.set_style(self.style.clone());

        self.draw(ui);
        self.handle_keys(ctx);
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        match self.state {
            State::Input => self.draw_input(ui),
            State::LoadingFirstLink => todo!(),
            State::Graph => todo!(),
            State::InputError => todo!(),
            State::LoadingError => todo!(),
            State::GraphAndLoading => todo!(),
        }
    }

    fn draw_input(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(self.size_section);

            ui.label(header_accent(HEADING));
            ui.add_space(self.size_margin);
            ui.label(DESCRIPTION);

            ui.add_space(self.size_section);
            ui.label(TOOLTIP);

            ui.add_space(self.size_margin);
            TextEdit::singleline(&mut self.root_article_url)
                .frame(false)
                .desired_rows(1)
                .vertical_align(Align::Center)
                .font(FontId::new(24., FontFamily::Monospace))
                .horizontal_align(Align::Center)
                .desired_width(f32::INFINITY)
                .show(ui)
                .response
                .request_focus();
        });
    }

    fn handle_keys(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Enter) {
                todo!();
            };
        });
    }
}

pub fn header_accent(text: &str) -> impl Into<WidgetText> {
    let mut job = LayoutJob::default();
    job.append(
        HEADING,
        0.0,
        TextFormat {
            font_id: FontId::new(24., FontFamily::Monospace),
            color: COLOR_ACCENT,
            ..Default::default()
        },
    );
    WidgetText::from(job)
}
