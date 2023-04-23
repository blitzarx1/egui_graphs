use egui::{Id, Vec2};

#[derive(Clone)]
pub struct Metadata {
    /// Current zoom factor
    pub zoom: f32,
    /// Current pan offset
    pub pan: Vec2,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            zoom: 1.,
            pan: Default::default(),
        }
    }
}

impl Metadata {
    pub fn get(ui: &mut egui::Ui) -> Self {
        ui.data_mut(|data| data.get_persisted::<Metadata>(Id::null()).unwrap_or_default())
    }

    pub fn store(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::null(), self);
        });
    }
}
