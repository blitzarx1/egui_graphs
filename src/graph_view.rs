use std::{
    f32::{MAX, MIN},
    sync::mpsc::Sender,
};

use crate::{
    changes::Changes,
    elements::{Elements, Node},
    metadata::Metadata,
    settings::SettingsInteraction,
    state::FrameState,
    Edge, SettingsNavigation,
};
use egui::{
    epaint::{CircleShape, CubicBezierShape, QuadraticBezierShape},
    Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget,
};

const ARROW_ANGLE: f32 = std::f32::consts::TAU / 50.;

/// GraphView is a widget for visualizing and interacting with graphs.
///
/// It implements egui::Widget and can be used like any other widget.
///
/// The widget uses a reference to the Elements struct to visualize the graph. You can
/// customize the visualization and interaction behavior using SettingsInteraction and
/// SettingsNavigation structs.
///
/// When any interaction supported by the widget occurs, it does not modify the provided Elements;
/// instead, it sends a Changes struct to the provided Sender<Changes> channel, which can be set via
/// the with_interactions method. It is up to the user to apply the changes to the Elements struct.
///
/// When the user performs navigation actions (zoom & pan, fit to screen), they do not
/// produce changes. This is because these actions are performed on the global coordinates and do not change
/// the position or scale of the graph elements.
#[derive(Clone)]
pub struct GraphView<'a> {
    elements: &'a Elements,
    setings_interaction: SettingsInteraction,
    setings_navigation: SettingsNavigation,
    changes_sender: Option<&'a Sender<Changes>>,
}

impl<'a> Widget for GraphView<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut metadata = Metadata::get(ui);

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        self.fit_if_first(&response, &mut metadata);

        let state = self.draw_and_sync(&painter, &mut metadata);

        self.handle_drags(&response, &state, &mut metadata);
        self.handle_click(&response, &state, &mut metadata);
        self.handle_navigation(ui, &response, &state, &mut metadata);

        metadata.store(ui);
        ui.ctx().request_repaint();

        response
    }
}

impl<'a> GraphView<'a> {
    /// Creates a new `GraphView` widget with default navigation and interactions settings.
    /// To customize navigation and interactions use `with_interactions` and `with_navigations` methods.
    pub fn new(elements: &'a Elements) -> Self {
        Self {
            elements,

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
        changes_sender: &'a Sender<Changes>,
    ) -> Self {
        self.changes_sender = Some(changes_sender);
        self.setings_interaction = settings_interaction.clone();
        self
    }

    /// Modifies default behaviour of navigation settings.
    pub fn with_navigations(mut self, settings_navigation: &SettingsNavigation) -> Self {
        self.setings_navigation = settings_navigation.clone();
        self
    }

    /// Resets navigation metadata
    pub fn reset_metadata(ui: &mut Ui) {
        Metadata::default().store(ui);
    }

    /// Fits the graph to the screen if it is the first frame
    fn fit_if_first(&self, r: &Response, m: &mut Metadata) {
        if !m.first_frame {
            return;
        }

        m.graph_bounds = compute_graph_bounds(self.elements);
        self.fit_to_screen(&r.rect, m);
        m.first_frame = false;
    }

    fn node_by_pos(&self, metadata: &Metadata, pos: Pos2) -> Option<(&usize, &'a Node)> {
        // transform pos to graph coordinates
        let pos_in_graph = (pos - metadata.pan).to_vec2() / metadata.zoom;
        let node_props = self
            .elements
            .get_nodes()
            .iter()
            .find(|(_, n)| (n.location - pos_in_graph).length() <= n.radius);

        node_props
    }

    fn send_changes(&self, changes: Changes) {
        if let Some(sender) = self.changes_sender {
            sender.send(changes).unwrap();
        }
    }

    fn handle_click(&self, response: &Response, state: &FrameState, metadata: &mut Metadata) {
        if !response.clicked() {
            return;
        }

        if !(self.setings_interaction.node_click
            || self.setings_interaction.node_select
            || self.setings_interaction.node_multiselect)
        {
            return;
        }

        // click on empty space
        let node = self.node_by_pos(metadata, response.hover_pos().unwrap());
        if node.is_none() && self.setings_interaction.node_select {
            self.deselect_all_nodes(state);
            self.deselect_all_edges(state);
            return;
        }

        let (idx, _) = node.unwrap();
        self.handle_node_click(idx, state);
    }

    fn handle_node_click(&self, idx: &usize, state: &FrameState) {
        if !self.setings_interaction.node_select {
            self.click_node(idx);
            return;
        }

        let n = self.elements.get_node(idx).unwrap();
        if n.selected {
            self.deselect_node(idx, n);
            return;
        }

        if !self.setings_interaction.node_multiselect {
            self.deselect_all_nodes(state);
            self.deselect_all_edges(state);
        }

        self.select_node(idx, n);
    }

    fn click_node(&self, idx: &usize) {
        let mut changes = Changes::default();
        changes.click_node(idx);
        self.send_changes(changes);
    }

    fn select_node(&self, idx: &usize, node: &Node) {
        let mut changes = Changes::default();
        changes.select_node(idx, node);
        self.send_changes(changes);
    }

    fn deselect_node(&self, idx: &usize, node: &Node) {
        let mut changes = Changes::default();
        changes.deselect_node(idx, node);
        self.send_changes(changes);
    }

    fn deselect_all_nodes(&self, state: &FrameState) {
        let mut changes = Changes::default();
        state.selected_nodes().iter().for_each(|idx| {
            let n = self.elements.get_node(idx).unwrap();
            changes.deselect_node(idx, n);
        });
        self.send_changes(changes);
    }

    fn deselect_all_edges(&self, state: &FrameState) {
        let mut changes = Changes::default();
        state.selected_edges().iter().for_each(|idx| {
            let e = self.elements.get_edge(idx).unwrap();
            changes.deselect_edge(idx, e);
        });
        self.send_changes(changes);
    }

    fn set_dragged_node(&self, idx: &usize) {
        let n = self.elements.get_node(idx).unwrap();
        let mut changes = Changes::default();
        changes.set_dragged_node(idx, n);
        self.send_changes(changes);
    }

    fn unset_dragged_node(&self, state: &FrameState) {
        if let Some(idx) = state.dragged_node() {
            let n = self.elements.get_node(&idx).unwrap();
            let mut changes = Changes::default();
            changes.unset_dragged_node(&idx, n);
            self.send_changes(changes);
        }
    }

    fn move_node(&self, idx: &usize, delta: Vec2) {
        let n = self.elements.get_node(idx).unwrap();
        let mut changes = Changes::default();
        changes.move_node(idx, n, delta);
        self.send_changes(changes);
    }

    fn handle_drags(&self, response: &Response, state: &FrameState, metadata: &mut Metadata) {
        if !self.setings_interaction.node_drag {
            return;
        }

        if response.drag_started() {
            if let Some((idx, _)) = self.node_by_pos(metadata, response.hover_pos().unwrap()) {
                self.set_dragged_node(idx);
            }
        }

        if response.dragged() && state.dragged_node().is_some() {
            let node_idx_dragged = state.dragged_node().unwrap();
            let delta_in_graph_coords = response.drag_delta() / metadata.zoom;
            self.move_node(&node_idx_dragged, delta_in_graph_coords);
        }

        if response.drag_released() && state.dragged_node().is_some() {
            self.unset_dragged_node(state);
        }
    }

    fn fit_to_screen(&self, rect: &Rect, metadata: &mut Metadata) {
        // calculate graph dimensions with decorative padding
        let diag = metadata.graph_bounds.max - metadata.graph_bounds.min;
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
        let zoom_delta = new_zoom / metadata.zoom - 1.0;
        let (new_zoom, _) = self.handle_zoom(rect, zoom_delta, None, metadata);

        // calculate the center of the graph and the canvas
        let graph_center =
            (metadata.graph_bounds.min.to_vec2() + metadata.graph_bounds.max.to_vec2()) / 2.0;

        // adjust the pan value to align the centers of the graph and the canvas
        (metadata.zoom, metadata.pan) =
            (new_zoom, rect.center().to_vec2() - graph_center * new_zoom)
    }

    fn handle_navigation(
        &self,
        ui: &Ui,
        response: &Response,
        state: &FrameState,
        metadata: &mut Metadata,
    ) {
        if self.setings_navigation.fit_to_screen {
            return self.fit_to_screen(&response.rect, metadata);
        }
        if !self.setings_navigation.zoom_and_pan {
            return;
        }

        let (mut new_zoom, mut new_pan) = (metadata.zoom, metadata.pan);
        ui.input(|i| {
            let delta = i.zoom_delta();
            if delta == 1. {
                return;
            }
            let step = self.setings_navigation.zoom_step * (1. - delta).signum();
            (new_zoom, new_pan) =
                self.handle_zoom(&response.rect, step, i.pointer.hover_pos(), metadata);
        });

        if response.dragged() && state.dragged_node().is_none() {
            (new_zoom, new_pan) = (new_zoom, metadata.pan + response.drag_delta());
        }

        (metadata.zoom, metadata.pan) = (new_zoom, new_pan);
    }

    fn handle_zoom(
        &self,
        rect: &Rect,
        delta: f32,
        zoom_center: Option<Pos2>,
        state: &Metadata,
    ) -> (f32, Vec2) {
        let center_pos = match zoom_center {
            Some(center_pos) => center_pos - rect.min,
            None => rect.center() - rect.min,
        };
        let graph_center_pos = (center_pos - state.pan) / state.zoom;
        let factor = 1. + delta;
        let new_zoom = state.zoom * factor;

        (
            new_zoom,
            state.pan + (graph_center_pos * state.zoom - graph_center_pos * new_zoom),
        )
    }

    fn draw_and_sync(&self, p: &Painter, metadata: &mut Metadata) -> FrameState {
        let mut state = FrameState::default();

        let edges_shapes = self.draw_and_sync_edges(p, &mut state, metadata);
        let nodes_shapes = self.draw_and_sync_nodes(p, &mut state, metadata);

        self.draw_edges_shapes(p, edges_shapes);
        self.draw_nodes_shapes(p, nodes_shapes);

        state
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

    fn draw_and_sync_edges(
        &self,
        p: &Painter,
        state: &mut FrameState,
        metadata: &Metadata,
    ) -> (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>) {
        let mut shapes = (Vec::new(), Vec::new(), Vec::new());
        self.elements.get_edges().iter().for_each(|(idx, edges)| {
            let mut order = edges.len();
            edges.iter().enumerate().for_each(|(list_idx, e)| {
                order -= 1;

                let edge = e.screen_transform(metadata.zoom);

                let edge_idx = (idx.0, idx.1, list_idx);
                GraphView::sync_edge(state, &edge_idx, e);

                let start_node = self.elements.get_node(&edge.start).unwrap();
                let end_node = self.elements.get_node(&edge.end).unwrap();
                let selected =
                    GraphView::draw_edge(&edge, p, start_node, end_node, metadata, order);
                selected.0.into_iter().for_each(|shape| {
                    shapes.0.push(shape);
                });
                selected.1.into_iter().for_each(|shape| {
                    shapes.1.push(shape);
                });
                selected.2.into_iter().for_each(|shape| {
                    shapes.2.push(shape);
                });
            });
        });
        shapes
    }

    fn sync_edge(state: &mut FrameState, idx: &(usize, usize, usize), e: &Edge) {
        if e.selected {
            state.select_edge(*idx);
        }
    }

    fn draw_edge(
        edge: &Edge,
        p: &Painter,
        n_start: &Node,
        n_end: &Node,
        metadata: &Metadata,
        order: usize,
    ) -> (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>) {
        let start_node = n_start.screen_transform(metadata.zoom, metadata.pan);
        let end_node = n_end.screen_transform(metadata.zoom, metadata.pan);

        let mut selected_shapes = vec![];
        let mut selected_quadratic = vec![];
        let mut selected_cubic = vec![];

        if edge.start == edge.end {
            GraphView::draw_edge_looped(p, &start_node, edge, order)
                .into_iter()
                .for_each(|c| {
                    selected_cubic.push(c);
                });
        }

        let shapes = GraphView::draw_edge_basic(p, &start_node, &end_node, edge, order);
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

    fn draw_edge_looped(p: &Painter, n: &Node, e: &Edge, order: usize) -> Vec<CubicBezierShape> {
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

        let stroke = Stroke::new(e.width, e.color);
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
        let highlighted_stroke = Stroke::new(
            e.width * 2.,
            Color32::from_rgba_unmultiplied(255, 0, 255, 128),
        );
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
        p: &Painter,
        n_start: &Node,
        n_end: &Node,
        e: &Edge,
        order: usize,
    ) -> (Vec<Shape>, Vec<QuadraticBezierShape>) {
        let pos_start = n_start.location.to_pos2();
        let pos_end = n_end.location.to_pos2();

        let vec = pos_end - pos_start;
        let l = vec.length();
        let dir = vec / l;

        let start_node_radius_vec = Vec2::new(n_start.radius, n_start.radius) * dir;
        let end_node_radius_vec = Vec2::new(n_end.radius, n_end.radius) * dir;

        let tip_point = pos_start + vec - end_node_radius_vec;
        let start_point = pos_start + start_node_radius_vec;

        let stroke = Stroke::new(e.width, e.color);
        let highlighted_stroke = Stroke::new(
            e.width * 2.,
            Color32::from_rgba_unmultiplied(255, 0, 255, 128),
        );

        // draw straight edge
        if order == 0 {
            let mut shapes = vec![];
            let head_point_1 = tip_point - e.tip_size * rotate_vector(dir, ARROW_ANGLE);
            let head_point_2 = tip_point - e.tip_size * rotate_vector(dir, -ARROW_ANGLE);

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

        let arrow_tip_dir_1 = rotate_vector(tip_dir, ARROW_ANGLE) * tip_size;
        let arrow_tip_dir_2 = rotate_vector(tip_dir, -ARROW_ANGLE) * tip_size;

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

    fn draw_and_sync_nodes(
        &self,
        p: &Painter,
        state: &mut FrameState,
        metadata: &mut Metadata,
    ) -> Vec<CircleShape> {
        let mut shapes = vec![];
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);
        self.elements.get_nodes().iter().for_each(|(idx, n)| {
            // update graph bounds
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

            GraphView::sync_node(self, idx, state, n);
            GraphView::draw_node(p, n, metadata)
                .into_iter()
                .for_each(|shape| {
                    shapes.push(shape);
                });
        });

        metadata.graph_bounds =
            Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

        shapes
    }

    fn sync_node(&self, idx: &usize, state: &mut FrameState, node: &Node) {
        if node.dragged {
            state.set_dragged_node(*idx);
        }
        if node.selected {
            state.select_node(*idx);
        }
    }

    fn draw_node(p: &Painter, n: &Node, metadata: &Metadata) -> Vec<CircleShape> {
        let node = &n.screen_transform(metadata.zoom, metadata.pan);
        let loc = node.location.to_pos2();

        GraphView::draw_node_basic(loc, p, node)
            .into_iter()
            .chain(GraphView::draw_node_interacted(loc, node).into_iter())
            .collect()
    }

    fn draw_node_basic(loc: Pos2, p: &Painter, node: &Node) -> Vec<CircleShape> {
        if !(node.selected || node.dragged) {
            p.circle_filled(loc, node.radius, node.color);
            return vec![];
        }

        vec![CircleShape {
            center: loc,
            radius: node.radius,
            fill: node.color,
            stroke: Stroke::new(1., node.color),
        }]
    }

    fn draw_node_interacted(loc: Pos2, node: &Node) -> Vec<CircleShape> {
        if !(node.selected || node.dragged) {
            return vec![];
        }

        let mut shapes = vec![];
        let highlight_radius = node.radius * 1.5;

        // draw a border around the selected node
        let highlight_color = Color32::from_rgba_unmultiplied(255, 0, 255, 128);
        if node.selected {
            shapes.push(CircleShape {
                center: loc,
                radius: highlight_radius,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::new(node.radius, highlight_color),
            });
        };

        // draw a border around the dragged node
        let dragged_color = Color32::from_rgba_unmultiplied(255, 255, 255, 128);
        if node.dragged {
            shapes.push(CircleShape {
                center: loc,
                radius: highlight_radius,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::new(node.radius, dragged_color),
            });
        }
        shapes
    }
}

fn compute_graph_bounds(elements: &Elements) -> Rect {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);

    elements.get_nodes().iter().for_each(|(_, n)| {
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

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}
