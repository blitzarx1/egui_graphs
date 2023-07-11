use crossbeam::channel::{Receiver, Sender};
use egui::Ui;
use egui_graphs::{Change, Graph, SettingsInteraction, SettingsNavigation, SettingsStyle};
use petgraph::EdgeType;

const EDGE_WEIGHT: f32 = 0.05;

pub struct State<'a, N: Clone, E: Clone, Ty: EdgeType> {
    pub loading: bool,
    pub g: &'a mut Graph<N, E, Ty>,
    pub sender: Sender<Change>,
    pub receiver: Receiver<Change>,
}

pub fn draw_view_graph<N: Clone, E: Clone, Ty: EdgeType>(ui: &mut Ui, state: State<N, E, Ty>) {
    let mut w = egui_graphs::GraphView::new(state.g);
    match state.loading {
        true => {
            w = w.with_styles(&SettingsStyle::default().with_edge_radius_weight(EDGE_WEIGHT));
        }
        false => {
            w = w.with_interactions(
                &SettingsInteraction::default()
                    .with_selection_enabled(true)
                    .with_dragging_enabled(true)
                    .with_selection_depth(i32::MAX),
            );
            w = w.with_navigations(
                &SettingsNavigation::default()
                    .with_fit_to_screen_enabled(false)
                    .with_zoom_and_pan_enabled(true),
            );
            w = w.with_styles(&SettingsStyle::default().with_edge_radius_weight(EDGE_WEIGHT));
        }
    }
    ui.add(&mut w);
}
