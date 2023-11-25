use eframe::{run_native, App, CreationContext};
use egui::{Context, SidePanel, TextEdit};
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsStyle,
};
use petgraph::stable_graph::{NodeIndex, StableGraph};

pub struct BasicApp {
    g: Graph<(), ()>,
    label_input: String,
    selected_node: Option<NodeIndex>,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self {
            g: Graph::from(&g),
            label_input: String::default(),
            selected_node: Option::default(),
        }
    }

    fn read_data(&mut self) {
        if let Some((selected_idx, _)) = self.g.nodes_iter().find(|(_, n)| n.selected()) {
            self.selected_node = Some(selected_idx.clone());
            self.label_input = self.g.node(selected_idx).unwrap().label();
        }
    }

    fn render(&mut self, ctx: &Context) {
        SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Change Label");
            TextEdit::singleline(&mut self.label_input)
                .hint_text("select node")
                .show(ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let widget =
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                    .with_interactions(
                        &SettingsInteraction::default().with_node_selection_enabled(true),
                    )
                    .with_styles(&SettingsStyle::default().with_labels_always(true));
            ui.add(widget);
        });
    }

    fn update_data(&mut self) {
        if self.selected_node.is_none() {
            return;
        }

        let idx = self.selected_node.unwrap();
        if idx.index().to_string() == self.label_input {
            return;
        }

        self.g
            .node_mut(idx)
            .unwrap()
            .set_label(self.label_input.clone());
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
        Box::new(|cc| Box::new(BasicApp::new(cc))),
    )
    .unwrap();
}
