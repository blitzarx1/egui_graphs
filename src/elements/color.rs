use egui::Color32;

pub fn inverse(c: Color32) -> Color32 {
    // check if alpha is not zero
    if c.is_additive() {
        Color32::from_rgb(255 - c.r(), 255 - c.g(), 255 - c.b())
    } else {
        Color32::from_rgba_unmultiplied(255 - c.r(), 255 - c.g(), 255 - c.b(), c.a())
    }
}
