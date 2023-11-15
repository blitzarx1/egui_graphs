use egui::{Context, Painter};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

use crate::{settings::SettingsStyle, Graph, Metadata};

use super::layers::Layers;
use super::{DefaultEdgeShape, DefaultNodeShape, EdgeDisplay, NodeDisplay};

/// Contains all the data about current widget state which is needed for custom drawing functions.
pub struct DrawContext<'a, N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> {
    pub ctx: &'a Context,
    pub g: &'a Graph<N, E, Ty, Ix>,
    pub style: &'a SettingsStyle,
    pub meta: &'a Metadata,
}

pub struct Drawer<'a, N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> {
    p: Painter,
    ctx: &'a DrawContext<'a, N, E, Ty, Ix>,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> Drawer<'a, N, E, Ty, Ix> {
    pub fn new(p: Painter, ctx: &'a DrawContext<'a, N, E, Ty, Ix>) -> Self {
        Drawer { p, ctx }
    }

    pub fn draw(self) {
        let mut l = Layers::default();

        self.fill_layers_edges::<DefaultEdgeShape<Ix>>(&mut l);
        self.fill_layers_nodes::<DefaultNodeShape>(&mut l);

        l.draw(self.p)
    }

    fn fill_layers_nodes<D: NodeDisplay<N, E, Ty, Ix>>(&self, l: &mut Layers) {
        self.ctx.g.nodes_iter().for_each(|(_, n)| {
            let shapes = D::from(n.clone().clone()).shapes(self.ctx);
            match n.selected() || n.dragged() {
                true => shapes.into_iter().for_each(|s| l.add_top(s)),
                false => shapes.into_iter().for_each(|s| l.add(s)),
            }
        });
    }

    fn fill_layers_edges<D: EdgeDisplay<N, E, Ty, Ix>>(&self, l: &mut Layers) {
        self.ctx.g.edges_iter().for_each(|(_, e)| {
            let shapes = D::from(e.clone().clone()).shapes(self.ctx);
            match e.selected() {
                true => shapes.into_iter().for_each(|s| l.add_top(s)),
                false => shapes.into_iter().for_each(|s| l.add(s)),
            }
        });
    }
}
