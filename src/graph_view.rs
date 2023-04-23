use std::{
    cell::RefCell,
    collections::HashMap,
    f32::{MAX, MIN},
};

use crate::{
    changes::Changes,
    elements::{Elements, Node},
    settings::Settings,
    state::State,
    Edge,
};
use egui::{
    epaint::{CubicBezierShape, QuadraticBezierShape},
    Color32, Painter, Pos2, Response, Sense, Stroke, Ui, Vec2, Widget,
};

const SCREEN_PADDING: f32 = 0.3;
const ZOOM_STEP: f32 = 0.1;
const ARROW_ANGLE: f32 = std::f32::consts::TAU / 50.;

#[derive(Clone)]
pub struct GraphView<'a> {
    elements: &'a Elements,
    settings: &'a Settings,

    top_left_pos: Vec2,
    down_right_pos: Vec2,

    changes: RefCell<Changes>,
}

impl<'a> GraphView<'a> {
    pub fn new(elements: &'a Elements, settings: &'a Settings) -> Self {
        let (top_left_pos, down_right_pos) = get_bounds(elements);

        Self {
            elements,
            settings,

            top_left_pos,
            down_right_pos,

            changes: Default::default(),
        }
    }

    /// Returns changes from the last frame
    pub fn last_changes(&self) -> Changes {
        self.changes.borrow().to_owned()
    }

    /// Should be called to clear cached graph metadata, for example
    /// in case when you provide completely different `Elements` from the one
    /// in the last frame
    pub fn reset_state(ui: &mut Ui) {
        State::default().store(ui);
    }

    // TODO: optimize this full scan run with quadtree or similar.
    // need to modify `crate::elements::Elements` to store nodes in a quadtree
    // Is it really necessary?
    fn node_by_pos(&self, state: &State, pos: Pos2) -> Option<(usize, &'a Node)> {
        let node_props = self.elements.get_nodes().iter().find(|(_, n)| {
            let node = n.screen_transform(state.zoom, state.pan);
            (node.location - pos.to_vec2()).length() <= node.radius
        });

        if let Some((idx, node)) = node_props {
            Some((*idx, node))
        } else {
            None
        }
    }

    fn handle_clicks(&self, response: &Response, state: &mut State, changes: &mut Changes) {
        if !self.settings.node_select {
            return;
        }
        if !response.clicked() {
            return;
        }

        let node = self.node_by_pos(state, response.hover_pos().unwrap());
        if node.is_none() {
            self.deselect_all_nodes(state, changes);
            self.deselect_all_edges(state, changes);
            return;
        }

        let (idx, n) = node.unwrap();
        if n.selected {
            self.deselect_node(&idx, state, changes);
            return;
        }
        self.select_node(&idx, state, changes);
    }

    fn select_node(&self, idx: &usize, state: &mut State, changes: &mut Changes) {
        let n = self.elements.get_node(idx).unwrap();

        if !self.settings.node_multiselect && !state.selected_nodes().is_empty() {
            self.deselect_all_nodes(state, changes);
        }

        changes.select_node(idx, n);
        state.select_node(*idx);
    }

    fn deselect_node(&self, idx: &usize, state: &mut State, changes: &mut Changes) {
        let n = self.elements.get_node(idx).unwrap();
        changes.deselect_node(idx, n);
        state.deselect_node(*idx);
    }

    fn deselect_all_nodes(&self, state: &mut State, changes: &mut Changes) {
        state.selected_nodes().iter().for_each(|idx| {
            let n = self.elements.get_node(idx).unwrap();
            changes.deselect_node(idx, n);
        });
        state.deselect_all_nodes();
    }

    fn deselect_all_edges(&self, state: &mut State, changes: &mut Changes) {
        state.selected_edges().iter().for_each(|idx| {
            let e = self.elements.get_edge(idx).unwrap();
            changes.deselect_edge(idx, e);
        });
        state.deselect_all_edges();
    }

    fn handle_drags(&self, response: &Response, state: &mut State, changes: &mut Changes) {
        if !self.settings.node_drag {
            return;
        }

        if response.drag_started() {
            if let Some((idx, _)) = self.node_by_pos(state, response.hover_pos().unwrap()) {
                state.set_dragged_node(idx);
            }
        }

        if response.dragged() && state.get_dragged_node().is_some() {
            let node_idx_dragged = state.get_dragged_node().unwrap();
            let node_dragged = self.elements.get_node(&node_idx_dragged).unwrap();

            let delta_in_graph_coords = response.drag_delta() / state.zoom;
            changes.move_node(&node_idx_dragged, node_dragged, delta_in_graph_coords);
        }

        if response.drag_released() && state.get_dragged_node().is_some() {
            state.unset_dragged_node();
        }
    }

    fn fit_to_screen(&self, state: &mut State, bounds: (Vec2, Vec2)) {
        // calculate graph dimensions with decorative padding
        let diag = bounds.1 - bounds.0;
        let graph_size = diag * (1. + SCREEN_PADDING);
        let (width, height) = (graph_size.x, graph_size.y);

        // calculate canvas dimensions
        let canvas_size = state.canvas.size();
        let (canvas_width, canvas_height) = (canvas_size.x, canvas_size.y);

        // calculate zoom factors for x and y to fit the graph inside the canvas
        let zoom_x = canvas_width / width;
        let zoom_y = canvas_height / height;

        // choose the minimum of the two zoom factors to avoid distortion
        let new_zoom = zoom_x.min(zoom_y);

        // calculate the zoom delta and call handle_zoom to adjust the zoom factor
        let zoom_delta = new_zoom / state.zoom - 1.0;
        let (new_zoom, _) = self.handle_zoom(zoom_delta, None, state);

        // calculate the center of the graph and the canvas
        let graph_center = (bounds.0 + bounds.1) / 2.0;

        // adjust the pan value to align the centers of the graph and the canvas
        (state.zoom, state.pan) = (
            new_zoom,
            state.canvas.center().to_vec2() - graph_center * new_zoom,
        )
    }

    fn handle_navigation(&self, ui: &Ui, response: &Response, state: &mut State) {
        if self.settings.fit_to_screen {
            return self.fit_to_screen(state, (self.top_left_pos, self.down_right_pos));
        }
        if !self.settings.zoom_and_pan {
            return;
        }

        let (mut new_zoom, mut new_pan) = (state.zoom, state.pan);
        ui.input(|i| {
            let delta = i.zoom_delta();
            if delta == 1. {
                return;
            }
            let step = ZOOM_STEP * (1. - delta).signum();
            (new_zoom, new_pan) = self.handle_zoom(step, i.pointer.hover_pos(), state);
        });

        if response.dragged() && state.get_dragged_node().is_none() {
            (new_zoom, new_pan) = (new_zoom, state.pan + response.drag_delta());
        }

        (state.zoom, state.pan) = (new_zoom, new_pan);
    }

    fn handle_zoom(&self, delta: f32, zoom_center: Option<Pos2>, state: &State) -> (f32, Vec2) {
        let center_pos = match zoom_center {
            Some(center_pos) => center_pos - state.canvas.min,
            None => state.canvas.center() - state.canvas.min,
        };
        let graph_center_pos = (center_pos - state.pan) / state.zoom;
        let factor = 1. + delta;
        let new_zoom = state.zoom * factor;

        (
            new_zoom,
            state.pan + (graph_center_pos * state.zoom - graph_center_pos * new_zoom),
        )
    }

    fn draw(&self, p: &Painter, state: &State) {
        self.draw_edges(p, state);
        self.draw_nodes(p, state);
    }

    fn draw_edges(&self, p: &Painter, state: &State) {
        self.elements.get_edges().iter().for_each(|(_, edges)| {
            let count = edges.len();
            let mut edges_count = HashMap::with_capacity(count);

            edges.iter().for_each(|e| {
                let edge = e.screen_transform(state.zoom);

                self.draw_edge(&edge, p, state, &mut edges_count)
            });
        });
    }

    fn draw_edge(
        &self,
        edge: &Edge,
        p: &Painter,
        state: &State,
        edges_count: &mut HashMap<(usize, usize), usize>,
    ) {
        let start_node = self
            .elements
            .get_node(&edge.start)
            .unwrap()
            .screen_transform(state.zoom, state.pan);
        let end_node = self
            .elements
            .get_node(&edge.end)
            .unwrap()
            .screen_transform(state.zoom, state.pan);

        let pos_start = start_node.location.to_pos2();
        let pos_end = end_node.location.to_pos2();

        let stroke = Stroke::new(edge.width, edge.color);

        // draw self-loop
        if edge.start == edge.end {
            // CubicBezierShape for self-loop
            let control_point1 = Pos2::new(
                pos_start.x + start_node.radius * 4.,
                pos_start.y - start_node.radius * 4.,
            );
            let control_point2 = Pos2::new(
                pos_start.x - start_node.radius * 4.,
                pos_start.y - start_node.radius * 4.,
            );

            p.add(CubicBezierShape::from_points_stroke(
                [pos_start, control_point1, control_point2, pos_end],
                true,
                Color32::TRANSPARENT,
                stroke,
            ));
            return;
        }

        let vec = pos_end - pos_start;
        let l = vec.length();
        let dir = vec / l;

        let end_node_radius_vec = Vec2::new(end_node.radius, end_node.radius) * dir;
        let start_node_radius_vec = Vec2::new(start_node.radius, start_node.radius) * dir;

        let tip_point = pos_start + vec - end_node_radius_vec;
        let start_point = pos_start + start_node_radius_vec;

        p.line_segment([start_point, tip_point], stroke);
        p.line_segment(
            [
                tip_point,
                tip_point - edge.tip_size * rotate_vector(dir, ARROW_ANGLE),
            ],
            stroke,
        );
        p.line_segment(
            [
                tip_point,
                tip_point - edge.tip_size * rotate_vector(dir, -ARROW_ANGLE),
            ],
            stroke,
        );

        if edge.selected {
            let highlighted_stroke = Stroke::new(
                edge.width * 2.,
                Color32::from_rgba_unmultiplied(255, 0, 255, 128),
            );
            p.line_segment([start_point, tip_point], highlighted_stroke);
            p.line_segment(
                [
                    tip_point,
                    tip_point - edge.tip_size * rotate_vector(dir, ARROW_ANGLE),
                ],
                highlighted_stroke,
            );
            p.line_segment(
                [
                    tip_point,
                    tip_point - edge.tip_size * rotate_vector(dir, -ARROW_ANGLE),
                ],
                highlighted_stroke,
            );
        }

        //     let key = (edge.start, edge.end);
        //     edges_count.entry(key).or_insert_with(|| 0);
        //     let curve_scale = edges_count.get_mut(&key).unwrap();
        //     *curve_scale += 1;

        //     let dir_perpendicular = Vec2::new(-dir.y, dir.x);
        //     let center_point = (start_point + tip_point.to_vec2()).to_vec2() / 2.0;
        //     let control_point = (center_point
        //         + dir_perpendicular * edge.curve_size * *curve_scale as f32)
        //         .to_pos2();

        //     p.add(QuadraticBezierShape::from_points_stroke(
        //         [start_point, control_point, tip_point],
        //         false,
        //         Color32::TRANSPARENT,
        //         stroke,
        //     ));

        //     let tip_vec = control_point - tip_point;
        //     let tip_dir = tip_vec / tip_vec.length();
        //     let tip_size = edge.tip_size;

        //     let arrow_tip_dir1 = rotate_vector(tip_dir, ARROW_ANGLE) * tip_size;
        //     let arrow_tip_dir2 = rotate_vector(tip_dir, -ARROW_ANGLE) * tip_size;

        //     let arrow_tip_point1 = tip_point + arrow_tip_dir1;
        //     let arrow_tip_point2 = tip_point + arrow_tip_dir2;

        //     p.line_segment([tip_point, arrow_tip_point1], stroke);
        //     p.line_segment([tip_point, arrow_tip_point2], stroke);
    }

    fn draw_nodes(&self, p: &Painter, state: &State) {
        self.elements.get_nodes().iter().for_each(|(idx, n)| {
            let node = n.screen_transform(state.zoom, state.pan);
            GraphView::draw_node(p, idx, &node, state)
        });
    }

    fn draw_node(p: &Painter, idx: &usize, node: &Node, state: &State) {
        let loc = node.location.to_pos2();
        p.circle_filled(loc, node.radius, node.color);

        let (highlight_radius, highlight_stroke_width) = (node.radius * 1.5, node.radius);

        // draw a border around the dragged node
        if state.get_dragged_node().is_some() && state.get_dragged_node().unwrap() == *idx {
            p.circle_stroke(
                loc,
                highlight_radius,
                Stroke::new(
                    highlight_stroke_width,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 128),
                ),
            );
            return;
        }

        // draw a border around the selected node
        if node.selected {
            p.circle_stroke(
                loc,
                highlight_radius,
                Stroke::new(
                    highlight_stroke_width,
                    Color32::from_rgba_unmultiplied(255, 0, 255, 128),
                ),
            )
        };
    }
}

impl<'a> Widget for &GraphView<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut state = State::get(ui);
        let mut changes = Changes::default();

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        state.canvas = response.rect;

        self.handle_drags(&response, &mut state, &mut changes);
        self.handle_clicks(&response, &mut state, &mut changes);

        self.handle_navigation(ui, &response, &mut state);
        self.draw(&painter, &state);

        state.store(ui);
        ui.ctx().request_repaint();

        // TODO: pass to response or similar to avoid mutability and usage of refcell and implementing
        // widget for pointer
        *self.changes.borrow_mut() = changes;

        response
    }
}

fn get_bounds(elements: &Elements) -> (Vec2, Vec2) {
    let mut min_x: f32 = MAX;
    let mut min_y = MAX;
    let mut max_x = MIN;
    let mut max_y = MIN;

    elements.get_nodes().iter().for_each(|(_, n)| {
        if n.location.x < min_x {
            min_x = n.location.x;
        };
        if n.location.y < min_y {
            min_y = n.location.y;
        };
        if n.location.x > max_x {
            max_x = n.location.x;
        };
        if n.location.y > max_y {
            max_y = n.location.y;
        };
    });

    (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}
