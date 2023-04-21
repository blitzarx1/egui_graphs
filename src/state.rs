use egui::{Id, Pos2, Rect, Vec2};

#[derive(Clone)]
pub struct State {
    /// current zoom factor
    pub zoom: f32,
    /// current pan offset
    pub pan: Vec2,
    /// index of the node that is currently being dragged
    pub node_dragged: Option<usize>,
    /// current canvas dimensions
    pub canvas: Rect,
}

impl Default for State {
    fn default() -> Self {
        Self {
            zoom: 1.,
            pan: Default::default(),
            node_dragged: Default::default(),
            canvas: Rect::from_min_max(Pos2::default(), Pos2::default()),
        }
    }
}

impl State {
    pub fn get(ui: &mut egui::Ui) -> Self {
        ui.data_mut(|data| data.get_persisted::<State>(Id::null()).unwrap_or_default())
    }

    pub fn store(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::null(), self);
        });
    }
}
