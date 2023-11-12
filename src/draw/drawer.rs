use std::collections::HashMap;

use egui::Painter;
use petgraph::graph::IndexType;
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::{settings::SettingsStyle, Edge, Graph, Metadata};

use super::{custom::DrawContext, default_edges_draw, default_node_draw, layers::Layers};

/// Mapping for 2 nodes and all edges between them
type EdgeMap<'a, E, Ix> = HashMap<(NodeIndex<Ix>, NodeIndex<Ix>), Vec<&'a Edge<E>>>;

pub struct Drawer<'a, N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> {
    p: Painter,

    g: &'a Graph<N, E, Ty, Ix>,
    style: &'a SettingsStyle,
    meta: &'a Metadata,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> Drawer<'a, N, E, Ty, Ix> {
    pub fn new(
        p: Painter,
        g: &'a Graph<N, E, Ty, Ix>,
        style: &'a SettingsStyle,
        meta: &'a Metadata,
    ) -> Self {
        Drawer { g, p, style, meta }
    }

    pub fn draw(self) {
        let mut l = Layers::default();

        self.fill_layers_edges(&mut l);
        self.fill_layers_nodes(&mut l);

        l.draw(self.p)
    }

    fn fill_layers_nodes(&self, l: &mut Layers) {
        let state = &DrawContext {
            ctx: self.p.ctx(),
            g: self.g,
            meta: self.meta,
            style: self.style,
        };
        self.g
            .nodes_iter()
            .for_each(|(_, n)| default_node_draw(self.p.ctx(), n, state, l));
    }

    fn fill_layers_edges(&self, l: &mut Layers) {
        let mut edge_map: EdgeMap<E, Ix> = HashMap::new();

        self.g.edges_iter().for_each(|(idx, e)| {
            let (source, target) = self.g.edge_endpoints(idx).unwrap();
            // compute map with edges between 2 nodes
            edge_map.entry((source, target)).or_default().push(e);
        });

        let state = &DrawContext {
            ctx: self.p.ctx(),
            g: self.g,
            meta: self.meta,
            style: self.style,
        };

        edge_map.into_iter().for_each(|((start, end), edges)| {
            default_edges_draw(self.p.ctx(), (start, end), edges, state, l)
        });
    }
}
