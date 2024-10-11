use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::Context;
use egui_graphs::{
    random_graph, DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, LayoutHierarchical,
    LayoutRandom, LayoutStateHierarchical, LayoutStateRandom,
};
use petgraph::{stable_graph::DefaultIx, Directed};

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
}

impl LayoutsApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let settings = Settings {
            layout: Layout::Hierarchical,
            num_nodes: 25,
            num_edges: 25,
        };
        Self {
            settings: settings.clone(),
            g: random_graph(settings.num_nodes, settings.num_edges),
        }
    }

    fn clear_cache(&mut self, ui: &mut egui::Ui) {
        match self.settings.layout {
            Layout::Hierarchical => {
                GraphView::<
                    (),
                    (),
                    Directed,
                    DefaultIx,
                    DefaultNodeShape,
                    DefaultEdgeShape,
                    LayoutStateHierarchical,
                    LayoutHierarchical,
                >::clear_cache(ui);
            }
            Layout::Random => {
                GraphView::<
                    (),
                    (),
                    Directed,
                    DefaultIx,
                    DefaultNodeShape,
                    DefaultEdgeShape,
                    LayoutStateRandom,
                    LayoutRandom,
                >::clear_cache(ui);
            }
        };
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
                            self.clear_cache(ui);
                        };
                        if ui
                            .radio_value(&mut self.settings.layout, Layout::Random, "Random")
                            .changed()
                        {
                            self.clear_cache(ui);
                        };
                    });
                    ui.horizontal(|ui| {
                        ui.label("Number of nodes");
                        if ui
                            .add(egui::Slider::new(&mut self.settings.num_nodes, 1..=250))
                            .changed()
                        {
                            self.clear_cache(ui);
                            self.g = random_graph(self.settings.num_nodes, self.settings.num_edges);
                        };
                    });
                    ui.horizontal(|ui| {
                        ui.label("Number of edges");
                        if ui
                            .add(egui::Slider::new(&mut self.settings.num_edges, 1..=250))
                            .changed()
                        {
                            self.clear_cache(ui);
                            self.g = random_graph(self.settings.num_nodes, self.settings.num_edges);
                        };
                    });
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.settings.layout {
                Layout::Hierarchical => {
                    ui.add(&mut GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        LayoutStateHierarchical,
                        LayoutHierarchical,
                    >::new(&mut self.g));
                }
                Layout::Random => {
                    ui.add(&mut GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        LayoutStateRandom,
                        LayoutRandom,
                    >::new(&mut self.g));
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
