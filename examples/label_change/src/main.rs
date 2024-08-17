use eframe::{run_native, App, CreationContext};
use egui::{CentralPanel, Context, SidePanel, TextEdit};
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsStyle,
};
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};

pub struct BasicApp {
    g: Graph<(), ()>,
    label_input: String,
    selected_node: Option<NodeIndex>,
    selected_edge: Option<EdgeIndex>,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self {
            g: Graph::from(&g),
            label_input: String::default(),
            selected_node: Option::default(),
            selected_edge: Option::default(),
        }
    }

    fn read_data(&mut self) {
        if !self.g.selected_nodes().is_empty() {
            let idx = self.g.selected_nodes().first().unwrap();
            self.selected_node = Some(*idx);
            self.selected_edge = None;
            self.label_input = self.g.node(*idx).unwrap().label();
        }
        if !self.g.selected_edges().is_empty() {
            let idx = self.g.selected_edges().first().unwrap();
            self.selected_edge = Some(*idx);
            self.selected_node = None;
            self.label_input = self.g.edge(*idx).unwrap().label();
        }
    }

    fn render(&mut self, ctx: &Context) {
        SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Change Label");
            ui.add_enabled_ui(
                self.selected_node.is_some() || self.selected_edge.is_some(),
                |ui| {
                    TextEdit::singleline(&mut self.label_input)
                        .hint_text("select node or edge")
                        .show(ui)
                },
            );
            if ui.button("reset").clicked() {
                self.reset()
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            let widget =
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                    .with_interactions(
                        &SettingsInteraction::default()
                            .with_node_selection_enabled(true)
                            .with_edge_selection_enabled(true),
                    )
                    .with_styles(&SettingsStyle::default().with_labels_always(true));
            ui.add(widget);
        });
    }

    fn update_data(&mut self) {
        if self.selected_node.is_none() && self.selected_edge.is_none() {
            return;
        }

        if self.selected_node.is_some() {
            let idx = self.selected_node.unwrap();
            if idx.index().to_string() == self.label_input {
                return;
            }

            self.g
                .node_mut(idx)
                .unwrap()
                .set_label(self.label_input.clone());
        }

        if self.selected_edge.is_some() {
            let idx = self.selected_edge.unwrap();
            if idx.index().to_string() == self.label_input {
                return;
            }

            self.g
                .edge_mut(idx)
                .unwrap()
                .set_label(self.label_input.clone());
        }
    }

    fn reset(&mut self) {
        let g = generate_graph();
        *self = Self {
            g: Graph::from(&g),
            label_input: String::default(),
            selected_node: Option::default(),
            selected_edge: Option::default(),
        };
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.read_data();
        self.render(ctx);
        self.update_data();
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
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
    .unwrap();
}
