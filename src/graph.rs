use std::{
    collections::HashMap,
    f32::{MAX, MIN},
};

use crate::{
    elements::{Elements, Node},
    settings::Settings,
    state::State,
};
use egui::{
    epaint::{CubicBezierShape, QuadraticBezierShape},
    Color32, Painter, Pos2, Response, Sense, Stroke, Ui, Vec2, Widget,
};
use petgraph::EdgeType;

const SCREEN_PADDING: f32 = 0.3;
const ZOOM_STEP: f32 = 0.1;

pub struct Graph<'a, N: Clone, E: Clone, Ty: EdgeType> {
    g: &'a mut petgraph::stable_graph::StableGraph<N, E, Ty>,
    elements: &'a mut Elements,
    settings: &'a Settings,

    top_left_pos: Vec2,
    down_right_pos: Vec2,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Graph<'a, N, E, Ty> {
    pub fn new(
        g: &'a mut petgraph::stable_graph::StableGraph<N, E, Ty>,
        elements: &'a mut Elements,
        settings: &'a Settings,
    ) -> Self {
        let (top_left_pos, down_right_pos) = get_bounds(elements);

        Self {
            g,
            elements,
            settings,

            top_left_pos,
            down_right_pos,
        }
    }

    pub fn reset_state(ui: &mut Ui) {
        State::default().store(ui);
    }

    // fn handle_all_interactions(&mut self, ui: &Ui, response: &Response) {
    //     ui.input(|i| {
    //         let delta = i.zoom_delta();
    //         if delta == 1. {
    //             return;
    //         }
    //         let step = ZOOM_STEP * (1. - delta).signum();
    //         self.handle_zoom(step, i.pointer.hover_pos());
    //     });

    //     self.handle_drags(response);
    // }

    // fn handle_drags(&mut self, response: &Response) {
    //     // FIXME: use k-d tree to find the closest node, check if distance is less than radius
    //     if response.drag_started() {
    //         let node_props = self.nodes_props.iter().find(|(_, props)| {
    //             (props.position - response.hover_pos().unwrap().to_vec2()).length() <= props.radius
    //         });

    //         if let Some((idx, _)) = node_props {
    //             self.node_dragged = Some(*idx);
    //         }
    //     }

    //     if response.dragged() {
    //         match self.node_dragged {
    //             // if we are dragging a node, we should update its position in the graph
    //             Some(node_dragged) => {
    //                 let node_pos = self.nodes_props.get(&node_dragged).unwrap().position;

    //                 // here we should update position in the graph coordinates
    //                 // because on every tick we recalculate node positions assuming
    //                 // that they are in graph coordinates

    //                 // convert node position from screen to graph coordinates
    //                 let graph_node_pos = (node_pos - self.pan) / self.zoom;

    //                 // apply scaled drag translation
    //                 let graph_dragged_pos = graph_node_pos + response.drag_delta() / self.zoom;
    //             }
    //             // if we are not dragging a node, we should pan the graph
    //             None => self.pan += response.drag_delta(),
    //         };
    //     }

    //     if response.drag_released() {
    //         self.node_dragged = Default::default();
    //     }
    // }
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Widget for Graph<'a, N, E, Ty> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut state = State::get(ui);

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        state.canvas = response.rect;

        if self.settings.fit_to_screen {
            fit_to_screen(
                &mut state,
                self.elements,
                (self.top_left_pos, self.down_right_pos),
            );
        }

        to_screen_coords(&state, &mut self.elements.nodes);
        draw(&painter, self.elements);

        state.store(ui);

        ui.ctx().request_repaint();

        response
    }
}

fn get_bounds(elements: &Elements) -> (Vec2, Vec2) {
    let mut min_x: f32 = MAX;
    let mut min_y = MAX;
    let mut max_x = MIN;
    let mut max_y = MIN;

    elements.nodes.iter().for_each(|(_, n)| {
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

fn to_screen_coords(state: &State, nodes: &mut HashMap<usize, Node>) {
    nodes.iter_mut().for_each(|(_, n)| {
        n.location = n.location * state.zoom + state.pan;
    });
}

fn draw(p: &Painter, elements: &Elements) {
    draw_edges(p, elements);
    draw_nodes(p, &elements.nodes);
}

fn draw_edges(p: &Painter, elements: &Elements) {
    let angle = std::f32::consts::TAU / 50.;

    elements.edges.iter().for_each(|(_, edges)| {
        let edges_count = edges.len();
        let mut sames = HashMap::with_capacity(edges_count);

        edges.iter().for_each(|edge| {
            let start_node_props = elements.nodes.get(&edge.start).unwrap();
            let end_node_props = elements.nodes.get(&edge.end).unwrap();

            let pos_start = start_node_props.location.to_pos2();
            let pos_end = end_node_props.location.to_pos2();

            let stroke = Stroke::new(edge.width, edge.color);

            if edge.start == edge.end {
                // CubicBezierShape for self-loop
                let control_point1 = Pos2::new(
                    pos_start.x + start_node_props.radius * 4.,
                    pos_start.y - start_node_props.radius * 4.,
                );
                let control_point2 = Pos2::new(
                    pos_start.x - start_node_props.radius * 4.,
                    pos_start.y - start_node_props.radius * 4.,
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

            let end_node_radius_vec = Vec2::new(end_node_props.radius, end_node_props.radius) * dir;
            let start_node_radius_vec =
                Vec2::new(start_node_props.radius, start_node_props.radius) * dir;

            let tip_point = pos_start + vec - end_node_radius_vec;
            let start_point = pos_start + start_node_radius_vec;
            match edges_count > 1 {
                true => {
                    let pair = [edge.start, edge.end];
                    sames.entry(pair).or_insert_with(|| 0);
                    let curve_scale = sames.get_mut(&pair).unwrap();
                    *curve_scale += 1;

                    let dir_perpendicular = Vec2::new(-dir.y, dir.x);
                    let center_point = (start_point + tip_point.to_vec2()).to_vec2() / 2.0;
                    let control_point = (center_point
                        + dir_perpendicular * edge.curve_size * *curve_scale as f32)
                        .to_pos2();

                    p.add(QuadraticBezierShape::from_points_stroke(
                        [start_point, control_point, tip_point],
                        false,
                        Color32::TRANSPARENT,
                        stroke,
                    ));

                    let tip_vec = control_point - tip_point;
                    let tip_dir = tip_vec / tip_vec.length();
                    let tip_size = edge.tip_size;

                    let arrow_tip_dir1 = rotate_vector(tip_dir, angle) * tip_size;
                    let arrow_tip_dir2 = rotate_vector(tip_dir, -angle) * tip_size;

                    let arrow_tip_point1 = tip_point + arrow_tip_dir1;
                    let arrow_tip_point2 = tip_point + arrow_tip_dir2;

                    p.line_segment([tip_point, arrow_tip_point1], stroke);
                    p.line_segment([tip_point, arrow_tip_point2], stroke);
                }
                false => {
                    p.line_segment([start_point, tip_point], stroke);
                    p.line_segment(
                        [
                            tip_point,
                            tip_point - edge.tip_size * rotate_vector(dir, angle),
                        ],
                        stroke,
                    );
                    p.line_segment(
                        [
                            tip_point,
                            tip_point - edge.tip_size * rotate_vector(dir, -angle),
                        ],
                        stroke,
                    );
                }
            }
        });
    });
}

fn draw_nodes(p: &Painter, nodes: &HashMap<usize, Node>) {
    nodes.iter().for_each(|(_, n)| {
        p.circle_filled(n.location.to_pos2(), n.radius, n.color);
    });
}

fn fit_to_screen(state: &mut State, elements: &mut Elements, bounds: (Vec2, Vec2)) {
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
    handle_zoom(zoom_delta, None, state, elements);

    // calculate the center of the graph and the canvas
    let graph_center = (bounds.0 + bounds.1) / 2.0;

    // adjust the pan value to align the centers of the graph and the canvas
    state.pan = state.canvas.center().to_vec2() - graph_center * state.zoom;
}

fn handle_zoom(delta: f32, zoom_center: Option<Pos2>, state: &mut State, elements: &mut Elements) {
    let center_pos = match zoom_center {
        Some(center_pos) => center_pos - state.canvas.min,
        None => Vec2::ZERO,
    };
    let graph_center_pos = (center_pos - state.pan) / state.zoom;
    let factor = 1. + delta;
    let new_zoom = state.zoom * factor;

    elements
        .nodes
        .iter_mut()
        .for_each(|(_, n)| n.radius *= factor);
    elements.edges.iter_mut().for_each(|(_, edges)| {
        edges.iter_mut().for_each(|e| {
            e.width *= factor;
            e.tip_size *= factor;
            e.curve_size *= factor;
        });
    });

    state.pan = (1. - factor) * graph_center_pos * new_zoom;
    state.zoom = new_zoom;
}
