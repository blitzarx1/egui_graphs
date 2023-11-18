use std::marker::PhantomData;

use egui::{Context, Painter};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

use crate::{settings::SettingsStyle, Graph, Metadata};

use super::layers::Layers;
use super::{DisplayEdge, DisplayNode};

/// Contains all the data about current widget state which is needed for custom drawing functions.
pub struct DrawContext<'a> {
    pub ctx: &'a Context,
    pub style: &'a SettingsStyle,
    pub is_directed: bool,
    pub meta: &'a Metadata,
}

pub struct Drawer<'a, N, E, Ty, Ix, Nd, Ed>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
{
    p: Painter,
    ctx: &'a DrawContext<'a>,
    g: &'a mut Graph<N, E, Ty, Ix, Nd>,
    _marker: PhantomData<(Nd, Ed)>,
}

impl<'a, N, E, Ty, Ix, Nd, Ed> Drawer<'a, N, E, Ty, Ix, Nd, Ed>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
{
    pub fn new(p: Painter, g: &'a mut Graph<N, E, Ty, Ix, Nd>, ctx: &'a DrawContext<'a>) -> Self {
        Drawer {
            p,
            ctx,
            g,
            _marker: PhantomData,
        }
    }

    pub fn draw(mut self) {
        let mut l = Layers::default();

        self.fill_layers_edges(&mut l);
        self.fill_layers_nodes(&mut l);

        l.draw(self.p)
    }

    fn fill_layers_nodes(&mut self, l: &mut Layers) {
        self.g
            .g
            .node_indices()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|idx| {
                let n = self.g.node_mut(idx).unwrap();

                let shapes = n.display_mut().shapes(self.ctx);
                match n.selected() || n.dragged() {
                    true => shapes.into_iter().for_each(|s| l.add_top(s)),
                    false => shapes.into_iter().for_each(|s| l.add(s)),
                }
            });
    }

    fn fill_layers_edges(&mut self, l: &mut Layers) {
        self.g
            .g
            .edge_indices()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|idx| {
                let (idx_start, idx_end) = self.g.edge_endpoints(idx).unwrap();
                let start = self.g.node(idx_start).unwrap();
                let end = self.g.node(idx_end).unwrap();

                let e = self.g.edge_mut(idx).unwrap();

                let shapes =
                    Ed::from(e.clone().clone()).shapes(start.clone(), end.clone(), self.ctx);
                match e.selected() {
                    true => shapes.into_iter().for_each(|s| l.add_top(s)),
                    false => shapes.into_iter().for_each(|s| l.add(s)),
                }
            });
    }
}
