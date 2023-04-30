use egui::{Id, Rect, Vec2};

#[derive(Clone)]
pub struct Metadata {
    /// If the frame is the first one
    pub first_frame: bool,
    /// Current zoom factor
    pub zoom: f32,
    /// Current pan offset
    pub pan: Vec2,
    /// Top left node position in the graph
    pub top_left_pos: Vec2,
    /// Bottom right node position in the graph
    pub down_right_pos: Vec2,
    /// Stores the bounds of the graph
    pub graph_bounds: Rect,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            first_frame: true,
            zoom: 1.,
            pan: Default::default(),
            top_left_pos: Default::default(),
            down_right_pos: Default::default(),
            graph_bounds: Rect::from_two_pos(egui::Pos2::default(), egui::Pos2::default()),
        }
    }
}

impl Metadata {
    pub fn get(ui: &mut egui::Ui) -> Self {
        ui.data_mut(|data| {
            data.get_persisted::<Metadata>(Id::null())
                .unwrap_or_default()
        })
    }

    pub fn store(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::null(), self);
        });
    }
}
