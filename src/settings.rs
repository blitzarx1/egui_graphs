/// `Settings` is a struct that holds configuration options for the `Graph` widget.
///
/// It contains settings to enable or disable various interactions with the graph,
/// such as autofitting the graph to the screen and starting the simulation when a node is dragged.
///
/// # Examples
///
/// ```
/// use your_crate_name::settings::Settings;
///
/// let settings = Settings {
///     autofit: true,
///     simulation_drag: false,
/// };
/// ```
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
