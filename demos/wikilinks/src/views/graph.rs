use egui::Ui;
use egui_graphs::{Graph, SettingsInteraction, SettingsNavigation, SettingsStyle};
use petgraph::EdgeType;

const EDGE_WEIGHT: f32 = 0.05;

pub fn draw_view_graph<N: Clone, E: Clone, Ty: EdgeType>(
    g: &mut Graph<N, E, Ty>,
    ui: &mut Ui,
    loading: bool,
) {
    let mut w = egui_graphs::GraphView::new(g);
    match loading {
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
