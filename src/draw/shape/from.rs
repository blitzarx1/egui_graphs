
use crate::{Edge, Node};

use super::graph_display::{EdgeGraphDisplay, NodeGraphDisplay};

pub trait FromNode<N: Clone> {
    fn from_node(node: &Node<N>) -> Box<dyn NodeGraphDisplay>;
}

pub trait FromEdge<E: Clone> {
    fn from_edge(edge: &Edge<E>) -> Box<dyn EdgeGraphDisplay>;
}
