use petgraph::stable_graph::NodeIndex;

#[derive(Default, Debug, Clone)]
pub struct FrameState {
    pub selected: Vec<NodeIndex>,
    pub dragged: Option<NodeIndex>,
}
