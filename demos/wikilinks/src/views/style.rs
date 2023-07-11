use egui::{text::LayoutJob, Color32, FontFamily, FontId, TextFormat, WidgetText};

pub const COLOR_ACCENT: Color32 = Color32::from_rgb(128, 128, 255);
pub const COLOR_SUB_ACCENT: Color32 = Color32::from_rgb(104, 138, 232);

pub const COLOR_LEFT_LOW: Color32 = Color32::from_rgb(148, 174, 179);
pub const COLOR_RIGHT_LOW: Color32 = Color32::from_rgb(175, 145, 179);

pub const COLOR_ERROR: Color32 = Color32::from_rgb(255, 64, 64);

pub const FONT_INPUT: FontId = FontId::new(24., FontFamily::Monospace);

pub const CURSOR_WIDTH: f32 = 5.;

pub fn header_accent(text: &str) -> impl Into<WidgetText> {
    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(24., FontFamily::Monospace),
            color: COLOR_ACCENT,
            ..Default::default()
        },
    );
    WidgetText::from(job)
}
