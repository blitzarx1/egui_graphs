use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::Context;
use egui_graphs::{
    random_graph, to_graph, DefaultEdgeShape, DefaultNodeShape, Graph, GraphView,
    LayoutHierarchical, LayoutRandom, LayoutStateHierarchical, LayoutStateRandom,
};
use petgraph::{stable_graph::DefaultIx, Directed};
use rand::Rng;

#[derive(Clone, PartialEq)]
enum Layout {
    Hierarchical,
    Random,
}

#[derive(Clone)]
struct Settings {
    layout: Layout,
    num_nodes: usize,
    num_edges: usize,
}
pub struct LayoutsApp {
    settings: Settings,
    g: Graph,
    reset_cache: bool,
}

impl LayoutsApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let settings = Settings {
            layout: Layout::Hierarchical,
            num_nodes: 25,
            num_edges: 25,
        };
        let g = to_graph(&random_graph(settings.num_nodes, settings.num_edges));
        Self {
            g,
            settings: settings.clone(),
            reset_cache: false,
        }
    }
}

impl App for LayoutsApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .min_width(250.)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Layout");
                        if ui
                            .radio_value(
                                &mut self.settings.layout,
                                Layout::Hierarchical,
                                "Hierarchical",
                            )
                            .changed()
                        {
                            self.reset_cache = true;
                        };
                        if ui
                            .radio_value(&mut self.settings.layout, Layout::Random, "Random")
                            .changed()
                        {
                            self.reset_cache = true;
                        };
                    });
                    ui.horizontal(|ui| {
                        ui.label("Number of nodes");
                        let mut value = self.settings.num_nodes;
                        if ui.add(egui::Slider::new(&mut value, 1..=250)).changed() {
                            let delta = value as isize - self.settings.num_nodes as isize;
                            if delta > 0 {
                                for _ in 0..delta {
                                    self.g.add_node(());
                                }
                            } else {
                                for _ in 0..-delta {
                                    let idx = self.g.node_indices().last().unwrap();
                                    self.g.remove_node(idx);
                                }
                            }

                            self.settings.num_nodes = value;
                            self.settings.num_edges = self.g.edge_count();
                        };
                    });
                    ui.horizontal(|ui| {
                        ui.label("Number of edges");
                        let mut value = self.settings.num_edges;
                        if ui.add(egui::Slider::new(&mut value, 1..=250)).changed() {
                            let delta = value as isize - self.settings.num_edges as isize;
                            if delta > 0 {
                                for _ in 0..delta {
                                    let mut rng = rand::thread_rng();
                                    let start = self
                                        .g
                                        .node_indices()
                                        .nth(rng.gen_range(0..self.g.node_count()))
                                        .unwrap();
                                    let end = self
                                        .g
                                        .node_indices()
                                        .nth(rng.gen_range(0..self.g.node_count()))
                                        .unwrap();
                                    self.g.add_edge(start, end, ());
                                }
                            } else {
                                for _ in 0..-delta {
                                    let idx = self.g.edge_indices().last().unwrap();
                                    self.g.remove_edge(idx);
                                }
                            }

                            self.settings.num_edges = value;
                        };
                    });
                    ui.horizontal(|ui| {
                        if ui.button("redraw").changed() {
                            self.reset_cache = true;
                        };
                    });
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.settings.layout {
                Layout::Hierarchical => {
                    let w = &mut GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        LayoutStateHierarchical,
                        LayoutHierarchical,
                    >::new(&mut self.g);
                    if self.reset_cache {
                        w.clear_cache(ui);
                        self.reset_cache = false;
                    }
                    ui.add(w);
                }
                Layout::Random => {
                    let w =
                        &mut GraphView::<_, _, _, _, _, _, LayoutStateRandom, LayoutRandom>::new(
                            &mut self.g,
                        );
                    if self.reset_cache {
                        w.clear_cache(ui);
                        self.reset_cache = false;
                    }
                    ui.add(w);
                }
            };
        });
    }
}

fn main() {
    run_native(
        "egui_graphs_layouts_demo",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(LayoutsApp::new(cc)))),
    )
    .unwrap();
}
