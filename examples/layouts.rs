use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::Context;
use egui_graphs::{
    generate_random_graph, DefaultEdgeShape, DefaultGraphView, DefaultNodeShape, Graph, GraphView,
    LayoutForceDirected, LayoutHierarchical, LayoutStateForceDirected, LayoutStateHierarchical,
};
use petgraph::{stable_graph::DefaultIx, Directed};

#[derive(Clone, PartialEq)]
enum Layout {
    Hierarchical,
    Random,
    ForceDirected,
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
            g: generate_random_graph(settings.num_nodes, settings.num_edges),
        }
    }

    fn reset(&mut self, ui: &mut egui::Ui) {
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
                >::reset(ui);
            }
            Layout::ForceDirected => {
                GraphView::<
                    (),
                    (),
                    Directed,
                    DefaultIx,
                    DefaultNodeShape,
                    DefaultEdgeShape,
                    LayoutStateForceDirected,
                    LayoutForceDirected,
                >::reset(ui);
            }
            Layout::Random => {
                DefaultGraphView::reset(ui);
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
                            self.reset(ui);
                        };
                        if ui
                            .radio_value(
                                &mut self.settings.layout,
                                Layout::ForceDirected,
                                "ForceDirected",
                            )
                            .changed()
                        {
                            self.reset(ui);
                        };
                        if ui
                            .radio_value(&mut self.settings.layout, Layout::Random, "Random")
                            .changed()
                        {
                            self.reset(ui);
                        };
                    });
                    ui.horizontal(|ui| {
                        ui.label("Number of nodes");
                        if ui
                            .add(egui::Slider::new(&mut self.settings.num_nodes, 1..=250))
                            .changed()
                        {
                            self.reset(ui);
                            self.g = generate_random_graph(
                                self.settings.num_nodes,
                                self.settings.num_edges,
                            );
                        };
                    });
                    ui.horizontal(|ui| {
                        ui.label("Number of edges");
                        if ui
                            .add(egui::Slider::new(&mut self.settings.num_edges, 1..=250))
                            .changed()
                        {
                            self.reset(ui);
                            self.g = generate_random_graph(
                                self.settings.num_nodes,
                                self.settings.num_edges,
                            );
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
                Layout::ForceDirected => {
                    ui.add(&mut GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        LayoutStateForceDirected,
                        LayoutForceDirected,
                    >::new(&mut self.g));
                }
                Layout::Random => {
                    ui.add(&mut DefaultGraphView::new(&mut self.g));
                }
            };
        });
    }
}

fn main() {
    run_native(
        "layouts",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(LayoutsApp::new(cc)))),
    )
    .unwrap();
}
