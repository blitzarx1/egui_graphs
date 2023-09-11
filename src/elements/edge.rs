use egui::Color32;

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Edge<E: Clone> {
    /// Client data
    data: Option<E>,

    width: f32,
    tip_size: f32,
    tip_angle: f32,
    curve_size: f32,

    /// If `color` is None default color is used.
    color: Option<Color32>,
}

impl<E: Clone> Default for Edge<E> {
    fn default() -> Self {
        Self {
            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 30.,
            curve_size: 20.,

            data: Default::default(),
            color: Default::default(),
        }
    }
}

impl<E: Clone> Edge<E> {
    pub fn new(data: E) -> Self {
        Self {
            data: Some(data),

            ..Default::default()
        }
    }

    pub fn tip_angle(&self) -> f32 {
        self.tip_angle
    }

    pub fn data(&self) -> Option<&E> {
        self.data.as_ref()
    }

    pub fn color(&self) -> Option<Color32> {
        self.color
    }

    pub fn with_color(&mut self, color: Color32) -> Self {
        let mut res = self.clone();
        res.color = Some(color);
        res
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn with_width(&mut self, width: f32) -> Self {
        let mut res = self.clone();
        res.width = width;
        res
    }

    pub fn curve_size(&self) -> f32 {
        self.curve_size
    }

    pub fn tip_size(&self) -> f32 {
        self.tip_size
    }
}