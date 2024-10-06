use std::marker::PhantomData;

use egui::{Context, Painter, Shape};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

use crate::{
    layouts::{Layout, LayoutState},
    settings::SettingsStyle,
    Graph, Metadata,
};

use super::{DisplayEdge, DisplayNode};

/// Contains all the data about current widget state which is needed for custom drawing functions.
pub struct DrawContext<'a> {
    pub ctx: &'a Context,
    pub painter: &'a Painter,
    pub style: &'a SettingsStyle,
    pub is_directed: bool,
    pub meta: &'a Metadata,
}

pub struct Drawer<'a, N, E, Ty, Ix, Nd, Ed, S, L>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
    S: LayoutState,
    L: Layout<S>,
{
    ctx: &'a DrawContext<'a>,
    g: &'a mut Graph<N, E, Ty, Ix, Nd, Ed>,
    delayed: Vec<Shape>,

    _marker: PhantomData<(Nd, Ed, L, S)>,
}

impl<'a, N, E, Ty, Ix, Nd, Ed, S, L> Drawer<'a, N, E, Ty, Ix, Nd, Ed, S, L>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
    S: LayoutState,
    L: Layout<S>,
{
    pub fn new(g: &'a mut Graph<N, E, Ty, Ix, Nd, Ed>, ctx: &'a DrawContext<'a>) -> Self {
        Drawer {
            ctx,
            g,
            delayed: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn draw(mut self) {
        self.draw_edges();
        self.draw_nodes();
        self.draw_postponed();
    }

    fn draw_postponed(&mut self) {
        self.delayed.iter().for_each(|s| {
            self.ctx.painter.add(s.clone());
        });
    }

    fn draw_nodes(&mut self) {
        self.g
            .g
            .node_indices()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|idx| {
                let n = self.g.node_mut(idx).unwrap();
                let props = n.props().clone();

                let display = n.display_mut();
                display.update(&props);
                let shapes = display.shapes(self.ctx);

                if n.selected() || n.dragged() {
                    for s in shapes {
                        self.delayed.push(s);
                    }
                } else {
                    for s in shapes {
                        self.ctx.painter.add(s);
                    }
                }
            });
    }

    fn draw_edges(&mut self) {
        self.g
            .g
            .edge_indices()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|idx| {
                let (idx_start, idx_end) = self.g.edge_endpoints(idx).unwrap();

                // FIXME: not a good decision to clone nodes for every edge
                let start = self.g.node(idx_start).cloned().unwrap();
                let end = self.g.node(idx_end).cloned().unwrap();

                let e = self.g.edge_mut(idx).unwrap();
                let props = e.props().clone();

                let display = e.display_mut();
                display.update(&props);
                let shapes = display.shapes(&start, &end, self.ctx);

                if e.selected() {
                    for s in shapes {
                        self.delayed.push(s);
                    }
                } else {
                    for s in shapes {
                        self.ctx.painter.add(s);
                    }
                }
            });
    }
}
