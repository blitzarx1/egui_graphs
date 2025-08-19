use crate::{MAX_EDGE_COUNT, MAX_NODE_COUNT};
use egui::Ui;

pub struct GraphCountSliders {
    pub nodes: usize,
    pub edges: usize,
}

pub fn graph_count_sliders(
    ui: &mut Ui,
    mut v: GraphCountSliders,
    mut on_change: impl FnMut(i32, i32),
) {
    let mut delta_nodes: i32 = 0;
    let mut delta_edges: i32 = 0;

    ui.horizontal(|ui| {
        let start = v.nodes;
        ui.label("N");
        ui.add(egui::Slider::new(&mut v.nodes, 0..=MAX_NODE_COUNT));
        if ui
            .small_button("-10")
            .on_hover_text("Remove 10 nodes (M)")
            .clicked()
        {
            v.nodes = v.nodes.saturating_sub(10);
        }
        if ui
            .small_button("-1")
            .on_hover_text("Remove 1 node (N)")
            .clicked()
        {
            v.nodes = v.nodes.saturating_sub(1);
        }
        if ui
            .small_button("+1")
            .on_hover_text("Add 1 node (n)")
            .clicked()
        {
            v.nodes = (v.nodes + 1).min(MAX_NODE_COUNT);
        }
        if ui
            .small_button("+10")
            .on_hover_text("Add 10 nodes (m)")
            .clicked()
        {
            v.nodes = (v.nodes + 10).min(MAX_NODE_COUNT);
        }
        delta_nodes = if v.nodes >= start {
            i32::try_from(v.nodes - start).unwrap()
        } else {
            -i32::try_from(start - v.nodes).unwrap()
        };
    });

    ui.horizontal(|ui| {
        let start = v.edges;
        ui.label("E");
        ui.add(egui::Slider::new(&mut v.edges, 0..=MAX_EDGE_COUNT));
        if ui
            .small_button("-10")
            .on_hover_text("Remove 10 edges (R)")
            .clicked()
        {
            v.edges = v.edges.saturating_sub(10);
        }
        if ui
            .small_button("-1")
            .on_hover_text("Remove 1 edge (E)")
            .clicked()
        {
            v.edges = v.edges.saturating_sub(1);
        }
        if ui
            .small_button("+1")
            .on_hover_text("Add 1 edge (e)")
            .clicked()
        {
            v.edges = (v.edges + 1).min(MAX_EDGE_COUNT);
        }
        if ui
            .small_button("+10")
            .on_hover_text("Add 10 edges (r)")
            .clicked()
        {
            v.edges = (v.edges + 10).min(MAX_EDGE_COUNT);
        }
        delta_edges = if v.edges >= start {
            i32::try_from(v.edges - start).unwrap()
        } else {
            -i32::try_from(start - v.edges).unwrap()
        };
    });

    if delta_nodes != 0 || delta_edges != 0 {
        on_change(delta_nodes, delta_edges);
    }
}
