use eframe::{run_native, App, CreationContext, Frame};
use egui::{CentralPanel, Context, Label, Layout, SidePanel};
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsNavigation,
};
use petgraph::stable_graph::StableGraph;

pub struct BasicApp {
    g: Graph<(), ()>,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let available_width = ctx.available_rect().width();
        // SidePanel::left("left_panel")
        //     .default_width(available_width / 3.)
        //     .resizable(true)
        //     .show(ctx, |ui| {
        //         ui.child_ui(ui.max_rect(), Layout::default());
        //         ui.add(
        //             &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(
        //                 &mut self.g,
        //             )
        //             .with_navigations(
        //                 &SettingsNavigation::default()
        //                     .with_fit_to_screen_enabled(false)
        //                     .with_zoom_and_pan_enabled(true),
        //             )
        //             .with_interactions(
        //                 &SettingsInteraction::default()
        //                     .with_node_selection_multi_enabled(true)
        //                     .with_node_selection_enabled(true)
        //                     .with_edge_selection_enabled(true)
        //                     .with_edge_selection_multi_enabled(true)
        //                     .with_dragging_enabled(true),
        //             ),
        //         )
        //     });
        SidePanel::right("right_panel")
            .default_width(available_width / 3.)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add(
                    &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(
                        &mut self.g,
                    )
                    .with_navigations(
                        &SettingsNavigation::default()
                            // .with_fit_to_screen_enabled(false)
                            .with_zoom_and_pan_enabled(true),
                    )
                    .with_interactions(
                        &SettingsInteraction::default()
                            .with_node_selection_multi_enabled(true)
                            .with_node_selection_enabled(true)
                            .with_edge_selection_enabled(true)
                            .with_edge_selection_multi_enabled(true)
                            .with_dragging_enabled(true),
                    ),
                )
            });
        CentralPanel::default().show(ctx, |ui| {
            ui.add(
                Label::new("long long long long long long long"), //     &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                                                                  //         .with_navigations(
                                                                  //             &SettingsNavigation::default()
                                                                  //                 // .with_fit_to_screen_enabled(false)
                                                                  //                 .with_zoom_and_pan_enabled(true),
                                                                  //         )
                                                                  //         .with_interactions(
                                                                  //             &SettingsInteraction::default()
                                                                  //                 .with_node_selection_multi_enabled(true)
                                                                  //                 .with_node_selection_enabled(true)
                                                                  //                 .with_edge_selection_enabled(true)
                                                                  //                 .with_edge_selection_multi_enabled(true)
                                                                  //                 .with_dragging_enabled(true),
                                                                  //         ),
            )
        });
    }
}

fn generate_graph() -> StableGraph<(), ()> {
    let mut g = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Box::new(BasicApp::new(cc))),
    )
    .unwrap();
}
