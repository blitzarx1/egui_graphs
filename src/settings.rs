#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Settings {
    pub simulation_autofit: bool,
    pub simulation_drag: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            simulation_autofit: false,
            simulation_drag: true,
        }
    }
}
