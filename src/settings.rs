#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Settings {
    /// Simulation autofit disables all other interactions with the graph and fits the graph to the screen on every simulation fram update
    pub simulation_autofit: bool,

    /// Simulation drag starts the simulation when a node is dragged
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
