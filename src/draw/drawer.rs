use egui::{Context, Painter};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

use crate::{settings::SettingsStyle, Graph, Metadata};

use super::{default_edges_draw, layers::Layers};
use super::{DefaultNodeShape, NodeDisplay};

/// Contains all the data about current widget state which is needed for custom drawing functions.
pub struct DrawContext<'a, N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> {
    pub ctx: &'a Context,
    pub g: &'a Graph<N, E, Ty, Ix>,
    pub style: &'a SettingsStyle,
    pub meta: &'a Metadata,
}

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
        self.fill_layers_nodes::<DefaultNodeShape>(&mut l);

        l.draw(self.p)
    }

    fn fill_layers_nodes<D: NodeDisplay<N, Ix>>(&self, l: &mut Layers) {
        let ctx = &DrawContext {
            ctx: self.p.ctx(),
            g: self.g,
            meta: self.meta,
            style: self.style,
        };
        self.g.nodes_iter().for_each(|(_, n)| {
            let shapes = D::from(n.clone().clone()).shapes(ctx);
            match n.selected() || n.dragged() {
                true => shapes.into_iter().for_each(|s| l.add_top(s)),
                false => shapes.into_iter().for_each(|s| l.add(s)),
            }
        });
    }

    fn fill_layers_edges(&self, l: &mut Layers) {
        let ctx = &DrawContext {
            ctx: self.p.ctx(),
            g: self.g,
            meta: self.meta,
            style: self.style,
        };

        self.g
            .edges_iter()
            .for_each(|(_, e)| default_edges_draw(ctx, e, l));
    }
}
