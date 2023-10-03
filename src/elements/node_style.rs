#[derive(Clone, Debug)]
pub struct StyleNode {
    pub radius: f32,
}

impl Default for StyleNode {
    fn default() -> Self {
        Self { radius: 5. }
    }
}