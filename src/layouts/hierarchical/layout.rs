use serde::{Deserialize, Serialize};

use crate::{
    layouts::{Layout, LayoutState},
    DisplayEdge, DisplayNode, Graph,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {}

impl LayoutState for State {}

#[derive(Debug, Default)]
pub struct Hierarchical {
    state: State,
}

impl Layout<State> for Hierarchical {
    fn next<
        N: Clone,
        E: Clone,
        Ty: petgraph::EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    >(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    ) {
        todo!()
    }

    fn state(&self) -> State {
        todo!()
    }

    fn from_state(state: State) -> impl Layout<State> {
        Hierarchical { state }
    }
}
