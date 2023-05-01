use std::{
    f32::{MAX, MIN},
    sync::mpsc::Sender,
};

use crate::{
    changes::Changes,
    elements::Node,
    frame_state::FrameState,
    metadata::Metadata,
    settings::{SettingsInteraction, SettingsStyle},
    Edge, SettingsNavigation,
};
use egui::{
    epaint::{CircleShape, CubicBezierShape, QuadraticBezierShape},
    Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget,
};
use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    visit::IntoNodeReferences,
};

/// `GraphView` is a widget for visualizing and interacting with graphs.
///
/// It implements `egui::Widget` and can be used like any other widget.
///
/// The widget uses a mutable reference to the `petgraph::StableGraph<egui_graphs::Node<N>, egui_graphs::Edge<E>>`
/// struct to visualize and interact with the graph. `N` and `E` is arbitrary client data associated with nodes and edges.
/// You can customize the visualization and interaction behavior using `SettingsInteraction`, `SettingsNavigation` and `SettingsStyle` structs.
///
/// When any interaction or node propery change supported by the widget occurs, the widget sends `Changes` struct to the provided
/// `Sender<Changes>` channel, which can be set via the `with_interactions` method. The `Changes` struct contains information about
/// the changes that occured in the graph. Client can use this information to modify external state of the application if needed.
///
/// When the user performs navigation actions (zoom & pan, fit to screen), they do not
/// produce changes. This is because these actions are performed on the global coordinates and do not change any
/// properties of the nodes or edges.
pub struct GraphView<'a, N: Clone, E: Clone> {
    g: &'a mut StableGraph<Node<N>, Edge<E>>,
    setings_interaction: SettingsInteraction,
    setings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,
    changes_sender: Option<&'a Sender<Changes>>,
}

impl<'a, N: Clone, E: Clone> Widget for &mut GraphView<'a, N, E> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut meta = Metadata::get(ui);

        let (resp, p) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        self.fit_if_first(&resp, &mut meta);

        let mut state = self.draw_and_get_state(&p, &mut meta);

        self.handle_nodes_drags(&resp, &mut state, &mut meta);
        self.handle_click(&resp, &mut state, &mut meta);
        self.handle_navigation(ui, &resp, &state, &mut meta);

        meta.store(ui);
        ui.ctx().request_repaint();

        resp
    }
}

impl<'a, N: Clone, E: Clone> GraphView<'a, N, E> {
    /// Creates a new `GraphView` widget with default navigation and interactions settings.
    /// To customize navigation and interactions use `with_interactions` and `with_navigations` methods.
    pub fn new(g: &'a mut StableGraph<Node<N>, Edge<E>>) -> Self {
        Self {
            g,

            settings_style: Default::default(),
            setings_interaction: Default::default(),
            setings_navigation: Default::default(),
            changes_sender: Default::default(),
        }
    }

    /// Makes widget interactive sending changes. Events which
    /// are configured in `settings_interaction` are sent to the channel as soon as the occured.
    pub fn with_interactions(
        mut self,
        settings_interaction: &SettingsInteraction,
    ) -> Self {
        self.setings_interaction = settings_interaction.clone();
        self
    }

    pub fn with_changes(mut self, changes_sender: &'a Sender<Changes>) -> Self {
        self.changes_sender = Some(changes_sender);
        self
    }

    /// Modifies default behaviour of navigation settings.
    pub fn with_navigations(mut self, settings_navigation: &SettingsNavigation) -> Self {
        self.setings_navigation = settings_navigation.clone();
        self
    }

    pub fn with_styles(mut self, settings_style: &SettingsStyle) -> Self {
        self.settings_style = settings_style.clone();
        self
    }

    /// Resets navigation metadata
    pub fn reset_metadata(ui: &mut Ui) {
        Metadata::default().store(ui);
    }

    /// Gets rect in which graph is contained including node radius
    fn bounding_rect(&self) -> Rect {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);

        self.g.node_weights().for_each(|n| {
            let x_minus_rad = n.location.x - n.radius;
            if x_minus_rad < min_x {
                min_x = x_minus_rad;
            };

            let y_minus_rad = n.location.y - n.radius;
            if y_minus_rad < min_y {
                min_y = y_minus_rad;
            };

            let x_plus_rad = n.location.x + n.radius;
            if x_plus_rad > max_x {
                max_x = x_plus_rad;
            };

            let y_plus_rad = n.location.y + n.radius;
            if y_plus_rad > max_y {
                max_y = y_plus_rad;
            };
        });

        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    }

    fn node_by_pos(&self, metadata: &Metadata, pos: Pos2) -> Option<(NodeIndex, &Node<N>)> {
        // transform pos to graph coordinates
        let pos_in_graph = (pos - metadata.pan).to_vec2() / metadata.zoom;
        self.g
            .node_references()
            .find(|(_, n)| (n.location - pos_in_graph).length() <= n.radius)
    }

    /// Fits the graph to the screen if it is the first frame
    fn fit_if_first(&self, r: &Response, m: &mut Metadata) {
        if !m.first_frame {
            return;
        }

        m.graph_bounds = self.bounding_rect();
        self.fit_to_screen(&r.rect, m);
        m.first_frame = false;
    }

    fn handle_click(&mut self, resp: &Response, state: &mut FrameState<E>, meta: &mut Metadata) {
        if !resp.clicked() {
            return;
        }

        let clickable = self.setings_interaction.node_click
            || self.setings_interaction.node_select
            || self.setings_interaction.node_multiselect;

        if !(clickable) {
            return;
        }

        // click on empty space
        let node = self.node_by_pos(meta, resp.hover_pos().unwrap());
        if node.is_none() {
            let selectable =
                self.setings_interaction.node_select || self.setings_interaction.node_multiselect;
            if selectable {
                self.deselect_all_nodes(state);
            }
            return;
        }

        self.handle_node_click(node.unwrap().0, state);
    }

    fn handle_node_click(&mut self, idx: NodeIndex, state: &FrameState<E>) {
        if !self.setings_interaction.node_select {
            self.click_node(idx);
            return;
        }

        let n = self.g.node_weight(idx).unwrap();
        if n.selected {
            self.deselect_node(idx);
            return;
        }

        if !self.setings_interaction.node_multiselect {
            self.deselect_all_nodes(state);
        }

        self.select_node(idx);
    }

    fn handle_nodes_drags(
        &mut self,
        resp: &Response,
        state: &mut FrameState<E>,
        meta: &mut Metadata,
    ) {
        if !self.setings_interaction.node_drag {
            return;
        }

        if resp.drag_started() {
            if let Some((idx, _)) = self.node_by_pos(meta, resp.hover_pos().unwrap()) {
                self.set_dragged(idx, true);
            }
        }

        if resp.dragged() && state.dragged.is_some() {
            let n_idx_dragged = state.dragged.unwrap();
            let delta_in_graph_coords = resp.drag_delta() / meta.zoom;
            self.move_node(n_idx_dragged, delta_in_graph_coords);
        }

        if resp.drag_released() && state.dragged.is_some() {
            self.unset_dragged_node(state);
        }
    }

    fn fit_to_screen(&self, rect: &Rect, meta: &mut Metadata) {
        // calculate graph dimensions with decorative padding
        let diag = meta.graph_bounds.max - meta.graph_bounds.min;
        let graph_size = diag * (1. + self.setings_navigation.screen_padding);
        let (width, height) = (graph_size.x, graph_size.y);

        // calculate canvas dimensions
        let canvas_size = rect.size();
        let (canvas_width, canvas_height) = (canvas_size.x, canvas_size.y);

        // calculate zoom factors for x and y to fit the graph inside the canvas
        let zoom_x = canvas_width / width;
        let zoom_y = canvas_height / height;

        // choose the minimum of the two zoom factors to avoid distortion
        let new_zoom = zoom_x.min(zoom_y);

        // calculate the zoom delta and call handle_zoom to adjust the zoom factor
        let zoom_delta = new_zoom / meta.zoom - 1.0;
        self.zoom(rect, zoom_delta, None, meta);

        // calculate the center of the graph and the canvas
        let graph_center =
            (meta.graph_bounds.min.to_vec2() + meta.graph_bounds.max.to_vec2()) / 2.0;

        // adjust the pan value to align the centers of the graph and the canvas
        meta.pan = rect.center().to_vec2() - graph_center * new_zoom;
    }

    fn handle_navigation(
        &self,
        ui: &Ui,
        resp: &Response,
        state: &FrameState<E>,
        meta: &mut Metadata,
    ) {
        if self.setings_navigation.fit_to_screen {
            return self.fit_to_screen(&resp.rect, meta);
        }

        self.handle_zoom(ui, resp, meta);
        self.handle_pan(resp, state, meta);
    }

    fn handle_zoom(&self, ui: &Ui, resp: &Response, meta: &mut Metadata) {
        if !self.setings_navigation.zoom_and_pan {
            return;
        }

        ui.input(|i| {
            let delta = i.zoom_delta();
            if delta == 1. {
                return;
            }
            let step = self.setings_navigation.zoom_step * (1. - delta).signum();
            self.zoom(&resp.rect, step, i.pointer.hover_pos(), meta);
        });
    }

    fn handle_pan(&self, resp: &Response, state: &FrameState<E>, meta: &mut Metadata) {
        if !self.setings_navigation.zoom_and_pan {
            return;
        }

        if resp.dragged() && state.dragged.is_none() {
            meta.pan += resp.drag_delta();
        }
    }

    fn zoom(&self, rect: &Rect, delta: f32, zoom_center: Option<Pos2>, meta: &mut Metadata) {
        let center_pos = match zoom_center {
            Some(center_pos) => center_pos - rect.min,
            None => rect.center() - rect.min,
        };
        let graph_center_pos = (center_pos - meta.pan) / meta.zoom;
        let factor = 1. + delta;
        let new_zoom = meta.zoom * factor;

        meta.pan += graph_center_pos * meta.zoom - graph_center_pos * new_zoom;
        meta.zoom = new_zoom;
    }

    fn click_node(&self, idx: NodeIndex) {
        let mut changes = Changes::default();
        changes.set_clicked(idx, true);
        self.send_changes(changes);
    }

    fn select_node(&mut self, idx: NodeIndex) {
        self.g.node_weight_mut(idx).unwrap().selected = true;

        let mut changes = Changes::default();
        changes.set_selected(idx, true);
        self.send_changes(changes);
    }

    fn deselect_node(&mut self, idx: NodeIndex) {
        self.g.node_weight_mut(idx).unwrap().selected = false;

        let mut changes = Changes::default();
        changes.set_selected(idx, false);
        self.send_changes(changes);
    }

    fn deselect_all_nodes(&mut self, state: &FrameState<E>) {
        if state.selected.is_empty() {
            return;
        }

        let mut changes = Changes::default();
        state.selected.iter().for_each(|idx| {
            self.g.node_weight_mut(*idx).unwrap().selected = false;
            changes.set_selected(*idx, false);
        });
        self.send_changes(changes);
    }

    fn set_dragged(&mut self, idx: NodeIndex, val: bool) {
        self.g.node_weight_mut(idx).unwrap().dragged = val;

        let mut changes = Changes::default();
        changes.set_dragged(idx, val);
        self.send_changes(changes);
    }

    fn unset_dragged_node(&mut self, state: &FrameState<E>) {
        let n_idx = state.dragged.unwrap();
        let n = self.g.node_weight_mut(n_idx).unwrap();
        n.dragged = false;

        let mut changes = Changes::default();
        changes.set_dragged(n_idx, false);
        self.send_changes(changes);
    }

    fn move_node(&mut self, idx: NodeIndex, val: Vec2) {
        let n = self.g.node_weight_mut(idx).unwrap();
        n.location += val;

        let mut changes = Changes::default();
        changes.set_location(idx, n.location);
        self.send_changes(changes);
    }

    fn draw_and_get_state(&mut self, p: &Painter, metadata: &mut Metadata) -> FrameState<E> {
        let mut frame_state = FrameState::default();

        // reset node radius
        let default_radius = Node::new(Vec2::default(), ()).radius;
        self.g
            .node_weights_mut()
            .for_each(|n| n.radius = default_radius);

        let edges = frame_state.edges_by_nodes(self.g);
        edges.iter().for_each(|((start, end), edges)| {
            self.g
                .node_weight_mut(NodeIndex::new(*start))
                .unwrap()
                .radius += self.settings_style.edge_radius_weight * edges.len() as f32;
            self.g.node_weight_mut(NodeIndex::new(*end)).unwrap().radius +=
                self.settings_style.edge_radius_weight * edges.len() as f32;
        });

        let edges_shapes = self.draw_edges(p, &mut frame_state, metadata);
        let nodes_shapes = self.draw_nodes(p, metadata, &mut frame_state);

        self.draw_edges_shapes(p, edges_shapes);
        self.draw_nodes_shapes(p, nodes_shapes);

        frame_state
    }

    fn draw_nodes(
        &self,
        p: &Painter,
        meta: &mut Metadata,
        frame_state: &mut FrameState<E>,
    ) -> Vec<CircleShape> {
        let mut shapes = vec![];
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);
        self.g.node_references().for_each(|(idx, n)| {
            // update graph bounds on the fly
            // we shall account for the node radius
            // so that the node is fully visible

            let x_minus_rad = n.location.x - n.radius;
            if x_minus_rad < min_x {
                min_x = x_minus_rad;
            };

            let y_minus_rad = n.location.y - n.radius;
            if y_minus_rad < min_y {
                min_y = y_minus_rad;
            };

            let x_plus_rad = n.location.x + n.radius;
            if x_plus_rad > max_x {
                max_x = x_plus_rad;
            };

            let y_plus_rad = n.location.y + n.radius;
            if y_plus_rad > max_y {
                max_y = y_plus_rad;
            };

            if n.dragged {
                frame_state.dragged = Some(idx);
            }

            if n.selected {
                frame_state.selected.push(idx)
            }

            let selected = self.draw_node(p, n, meta);
            shapes.extend(selected);
        });

        meta.graph_bounds = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

        shapes
    }

    fn draw_edges(
        &mut self,
        p: &Painter,
        state: &mut FrameState<E>,
        meta: &Metadata,
    ) -> (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>) {
        let mut shapes = (Vec::new(), Vec::new(), Vec::new());
        state
            .edges_by_nodes(self.g)
            .iter()
            .for_each(|((start, end), edges)| {
                let mut order = edges.len();
                edges.iter().enumerate().for_each(|(_, e)| {
                    order -= 1;

                    let edge = e.screen_transform(meta.zoom);

                    let selected = self.draw_edge(&edge, p, start, end, meta, order);
                    shapes.0.extend(selected.0);
                    shapes.1.extend(selected.1);
                    shapes.2.extend(selected.2);
                });
            });

        shapes
    }

    fn draw_edges_shapes(
        &self,
        p: &Painter,
        shapes: (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>),
    ) {
        shapes.0.into_iter().for_each(|shape| {
            p.add(shape);
        });
        shapes.1.into_iter().for_each(|shape| {
            p.add(shape);
        });
        shapes.2.into_iter().for_each(|shape| {
            p.add(shape);
        });
    }

    fn draw_nodes_shapes(&self, p: &Painter, shapes: Vec<CircleShape>) {
        shapes.into_iter().for_each(|shape| {
            p.add(shape);
        });
    }

    fn draw_edge(
        &self,
        edge: &Edge<E>,
        p: &Painter,
        start: &usize,
        end: &usize,
        meta: &Metadata,
        order: usize,
    ) -> (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>) {
        let idx_start = NodeIndex::new(*start);
        let idx_end = NodeIndex::new(*end);

        let start_node = self
            .g
            .node_weight(idx_start)
            .unwrap()
            .screen_transform(meta.zoom, meta.pan);
        let end_node = self
            .g
            .node_weight(idx_end)
            .unwrap()
            .screen_transform(meta.zoom, meta.pan);

        let mut selected_shapes = vec![];
        let mut selected_quadratic = vec![];
        let mut selected_cubic = vec![];

        if start == end {
            self.draw_edge_looped(p, &start_node, edge, order)
                .into_iter()
                .for_each(|c| {
                    selected_cubic.push(c);
                });
        }

        let shapes = self.draw_edge_basic(p, &start_node, &end_node, edge, order);
        shapes
            .0
            .into_iter()
            .for_each(|shape| selected_shapes.push(shape));
        shapes
            .1
            .into_iter()
            .for_each(|shape| selected_quadratic.push(shape));

        (selected_shapes, selected_cubic, selected_quadratic)
    }

    fn draw_edge_looped(
        &self,
        p: &Painter,
        n: &Node<N>,
        e: &Edge<E>,
        order: usize,
    ) -> Vec<CubicBezierShape> {
        let color = match e.color {
            Some(color) => color,
            None => self.settings_style.color_edge(p.ctx()),
        };
        let pos_start_and_end = n.location.to_pos2();
        let loop_size = n.radius * (4. + 1. + order as f32);

        let control_point1 = Pos2::new(
            pos_start_and_end.x + loop_size,
            pos_start_and_end.y - loop_size,
        );
        let control_point2 = Pos2::new(
            pos_start_and_end.x - loop_size,
            pos_start_and_end.y - loop_size,
        );

        let stroke = Stroke::new(e.width, color);
        let shape_basic = CubicBezierShape::from_points_stroke(
            [
                pos_start_and_end,
                control_point1,
                control_point2,
                pos_start_and_end,
            ],
            true,
            Color32::TRANSPARENT,
            stroke,
        );

        if !e.selected {
            p.add(shape_basic);
            return vec![];
        }

        let mut shapes = vec![shape_basic];
        let highlighted_stroke = Stroke::new(e.width * 2., self.settings_style.color_highlight);
        shapes.push(CubicBezierShape::from_points_stroke(
            [
                pos_start_and_end,
                control_point1,
                control_point2,
                pos_start_and_end,
            ],
            true,
            Color32::TRANSPARENT,
            highlighted_stroke,
        ));

        shapes
    }

    fn draw_edge_basic(
        &self,
        p: &Painter,
        n_start: &Node<N>,
        n_end: &Node<N>,
        e: &Edge<E>,
        order: usize,
    ) -> (Vec<Shape>, Vec<QuadraticBezierShape>) {
        let color = match e.color {
            Some(color) => color,
            None => self.settings_style.color_edge(p.ctx()),
        };
        let pos_start = n_start.location.to_pos2();
        let pos_end = n_end.location.to_pos2();

        let vec = pos_end - pos_start;
        let l = vec.length();
        let dir = vec / l;

        let start_node_radius_vec = Vec2::new(n_start.radius, n_start.radius) * dir;
        let end_node_radius_vec = Vec2::new(n_end.radius, n_end.radius) * dir;

        let tip_point = pos_start + vec - end_node_radius_vec;
        let start_point = pos_start + start_node_radius_vec;

        let stroke = Stroke::new(e.width, color);
        let highlighted_stroke = Stroke::new(e.width * 2., self.settings_style.color_highlight);

        // draw straight edge
        if order == 0 {
            let mut shapes = vec![];
            let head_point_1 = tip_point - e.tip_size * rotate_vector(dir, e.tip_angle);
            let head_point_2 = tip_point - e.tip_size * rotate_vector(dir, -e.tip_angle);

            shapes.push(Shape::line_segment([start_point, tip_point], stroke));
            shapes.push(Shape::line_segment([tip_point, head_point_1], stroke));
            shapes.push(Shape::line_segment([tip_point, head_point_2], stroke));

            if !e.selected {
                shapes.into_iter().for_each(|shape| {
                    p.add(shape);
                });

                return (vec![], vec![]);
            }

            shapes.push(Shape::line_segment(
                [start_point, tip_point],
                highlighted_stroke,
            ));
            shapes.push(Shape::line_segment(
                [tip_point, head_point_1],
                highlighted_stroke,
            ));
            shapes.push(Shape::line_segment(
                [tip_point, head_point_2],
                highlighted_stroke,
            ));

            return (shapes, vec![]);
        }

        let mut shapes = vec![];
        let mut quadratic_shapes = vec![];
        let dir_perpendicular = Vec2::new(-dir.y, dir.x);
        let center_point = (start_point + tip_point.to_vec2()).to_vec2() / 2.0;
        let control_point =
            (center_point + dir_perpendicular * e.curve_size * order as f32).to_pos2();

        let tip_vec = control_point - tip_point;
        let tip_dir = tip_vec / tip_vec.length();
        let tip_size = e.tip_size;

        let arrow_tip_dir_1 = rotate_vector(tip_dir, e.tip_angle) * tip_size;
        let arrow_tip_dir_2 = rotate_vector(tip_dir, -e.tip_angle) * tip_size;

        let head_point_1 = tip_point + arrow_tip_dir_1;
        let head_point_2 = tip_point + arrow_tip_dir_2;

        quadratic_shapes.push(QuadraticBezierShape::from_points_stroke(
            [start_point, control_point, tip_point],
            false,
            Color32::TRANSPARENT,
            stroke,
        ));
        shapes.push(Shape::line_segment([tip_point, head_point_1], stroke));
        shapes.push(Shape::line_segment([tip_point, head_point_2], stroke));

        if !e.selected {
            quadratic_shapes.into_iter().for_each(|shape| {
                p.add(shape);
            });
            shapes.into_iter().for_each(|shape| {
                p.add(shape);
            });

            return (vec![], vec![]);
        }

        quadratic_shapes.push(QuadraticBezierShape::from_points_stroke(
            [start_point, control_point, tip_point],
            false,
            Color32::TRANSPARENT,
            highlighted_stroke,
        ));
        shapes.push(Shape::line_segment(
            [tip_point, head_point_1],
            highlighted_stroke,
        ));
        shapes.push(Shape::line_segment(
            [tip_point, head_point_2],
            highlighted_stroke,
        ));

        (shapes, quadratic_shapes)
    }

    fn draw_node(&self, p: &Painter, n: &Node<N>, meta: &Metadata) -> Vec<CircleShape> {
        let node = &n.screen_transform(meta.zoom, meta.pan);
        let loc = node.location.to_pos2();

        self.draw_node_basic(loc, p, node)
            .into_iter()
            .chain(self.draw_node_interacted(loc, node).into_iter())
            .collect()
    }

    fn draw_node_basic(&self, loc: Pos2, p: &Painter, node: &Node<N>) -> Vec<CircleShape> {
        let color = match node.color {
            Some(c) => c,
            None => self.settings_style.color_node(p.ctx()),
        };

        if !(node.selected || node.dragged) {
            p.circle_filled(loc, node.radius, color);
            return vec![];
        }

        vec![CircleShape {
            center: loc,
            radius: node.radius,
            fill: color,
            stroke: Stroke::new(1., color),
        }]
    }

    fn draw_node_interacted(&self, loc: Pos2, node: &Node<N>) -> Vec<CircleShape> {
        if !(node.selected || node.dragged) {
            return vec![];
        }

        let mut shapes = vec![];
        let highlight_radius = node.radius * 1.5;

        // draw a border around the selected node
        if node.selected {
            shapes.push(CircleShape {
                center: loc,
                radius: highlight_radius,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::new(node.radius, self.settings_style.color_highlight),
            });
        };

        // draw a border around the dragged node
        if node.dragged {
            shapes.push(CircleShape {
                center: loc,
                radius: highlight_radius,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::new(node.radius, self.settings_style.color_drag),
            });
        }
        shapes
    }

    fn send_changes(&self, changes: Changes) {
        if let Some(sender) = self.changes_sender {
            sender.send(changes).unwrap();
        }
    }
}

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::stable_graph::StableGraph;

    // Helper function to create a test StableGraph
    fn create_test_graph() -> StableGraph<Node<()>, Edge<usize>> {
        let mut graph = StableGraph::<Node<()>, Edge<usize>>::new();
        let n0 = graph.add_node(Node::new(Vec2::new(0.0, 0.0), ()));
        let n1 = graph.add_node(Node::new(Vec2::new(10.0, 10.0), ()));
        let n2 = graph.add_node(Node::new(Vec2::new(20.0, 20.0), ()));

        graph.add_edge(n0, n1, Edge::new(1));
        graph.add_edge(n0, n2, Edge::new(2));
        graph.add_edge(n1, n2, Edge::new(3));

        graph
    }

    #[test]
    fn test_bounding_rect() {
        let mut graph = create_test_graph();
        let graph_view = GraphView::<_, usize>::new(&mut graph);

        let bounding_rect = graph_view.bounding_rect();

        assert_eq!(bounding_rect.min, Pos2::new(-5.0, -5.0));
        assert_eq!(bounding_rect.max, Pos2::new(25.0, 25.0));
    }
}
