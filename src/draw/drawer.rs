use std::marker::PhantomData;

use egui::{Context, Id, LayerId, Order, Painter};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

use crate::{settings::SettingsStyle, Graph, Metadata};

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
    layer_top: Painter,
    layer_bot: Painter,
    ctx: &'a DrawContext<'a>,
    g: &'a mut Graph<N, E, Ty, Ix, Nd, Ed>,
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
    pub fn new(
        p: Painter,
        g: &'a mut Graph<N, E, Ty, Ix, Nd, Ed>,
        ctx: &'a DrawContext<'a>,
    ) -> Self {
        let layer_top = p
            .clone()
            .with_layer_id(LayerId::new(Order::Foreground, Id::new("top")));
        let layer_bot = p.with_layer_id(LayerId::background());
        Drawer {
            layer_top,
            layer_bot,
            ctx,
            g,
            _marker: PhantomData,
        }
    }

    pub fn draw(mut self) {
        self.draw_edges();
        self.fill_layers_nodes();
    }

    fn fill_layers_nodes(&mut self) {
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
                    shapes.into_iter().for_each(|s| {
                        self.layer_top.add(s);
                    });
                } else {
                    shapes.into_iter().for_each(|s| {
                        self.layer_bot.add(s);
                    });
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
                    shapes.into_iter().for_each(|s| {
                        self.layer_top.add(s);
                    });
                } else {
                    shapes.into_iter().for_each(|s| {
                        self.layer_bot.add(s);
                    });
                }
            });
    }
}
