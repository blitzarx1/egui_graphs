#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Settings {
    /// Autofit disables all other interactions with the graph and fits the graph to the screen on every simulation fram update
    pub autofit: bool,

    /// Simulation drag starts the simulation when a node is dragged
    pub simulation_drag: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autofit: true,
            simulation_drag: false,
        }
    }
}
