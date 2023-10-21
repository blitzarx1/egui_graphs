use std::collections::HashMap;

use egui::Painter;
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::{settings::SettingsStyle, Edge, Graph, Metadata};

use super::{
    custom::{FnCustomNodeDraw, WidgetState},
    default_edges_draw, default_node_draw,
    layers::Layers,
    FnCustomEdgeDraw,
};

/// Mapping for 2 nodes and all edges between them
type EdgeMap<'a, E> = HashMap<(NodeIndex, NodeIndex), Vec<&'a Edge<E>>>;

pub struct Drawer<'a, N: Clone, E: Clone, Ty: EdgeType> {
    p: Painter,

    g: &'a Graph<N, E, Ty>,
    style: &'a SettingsStyle,
    meta: &'a Metadata,

    custom_node_draw: Option<FnCustomNodeDraw<N, E, Ty>>,
    custom_edge_draw: Option<FnCustomEdgeDraw<N, E, Ty>>,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Drawer<'a, N, E, Ty> {
    pub fn new(
        p: Painter,
        g: &'a Graph<N, E, Ty>,
        style: &'a SettingsStyle,
        meta: &'a Metadata,
        custom_node_draw: Option<FnCustomNodeDraw<N, E, Ty>>,
        custom_edge_draw: Option<FnCustomEdgeDraw<N, E, Ty>>,
    ) -> Self {
        Drawer {
            g,
            p,
            style,
            meta,
            custom_node_draw,
            custom_edge_draw,
        }
    }

    pub fn draw(self) {
        let mut l = Layers::default();

        self.fill_layers_edges(&mut l);
        self.fill_layers_nodes(&mut l);

        l.draw(self.p)
    }

    fn fill_layers_nodes(&self, l: &mut Layers) {
        let state = &WidgetState {
            g: self.g,
            meta: self.meta,
            style: self.style,
        };
        self.g
            .nodes_iter()
            .for_each(|(_, n)| match self.custom_node_draw {
                Some(f) => f(self.p.ctx(), n, state, l),
                None => default_node_draw(self.p.ctx(), n, state, l),
            });
    }

    fn fill_layers_edges(&self, l: &mut Layers) {
        let mut edge_map: EdgeMap<E> = HashMap::new();

        self.g.edges_iter().for_each(|(idx, e)| {
            let (source, target) = self.g.edge_endpoints(idx).unwrap();
            // compute map with edges between 2 nodes
            edge_map.entry((source, target)).or_default().push(e);
        });

        let state = &WidgetState {
            g: self.g,
            meta: self.meta,
            style: self.style,
        };

        edge_map
            .into_iter()
            .for_each(|((start, end), edges)| match self.custom_edge_draw {
                Some(f) => f(self.p.ctx(), (start, end), edges, state, l),
                None => default_edges_draw(self.p.ctx(), (start, end), edges, state, l),
            });
    }
}
