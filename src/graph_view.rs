use std::{
    f32::{MAX, MIN},
    sync::{mpsc::Sender, Mutex},
};

use crate::{
    change::Change,
    change::ChangeNode,
    drawer::Drawer,
    elements::Node,
    graph_wrapper::GraphWrapper,
    metadata::Metadata,
    settings::{SettingsInteraction, SettingsStyle},
    state_computed::StateComputed,
    Edge, SettingsNavigation,
};
use egui::{Painter, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    EdgeType,
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
pub struct GraphView<'a, N: Clone, E: Clone, Ty: EdgeType> {
    settings_interaction: SettingsInteraction,
    setings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,
    g: GraphWrapper<'a, N, E, Ty>,
    changes_sender: Option<&'a Sender<Change>>,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Widget for &mut GraphView<'a, N, E, Ty> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (resp, p) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        let mut meta = Metadata::get(ui);

        let mut computed =
            StateComputed::compute(&self.g, &self.settings_interaction, &self.settings_style);
        // let mut guarded_computed = Mutex::new(&mut computed);
        // {
        //     let (n_walker, e_walker) = StateComputed::provide_compute_walkers::<N, E, Ty>(
        //         guarded_computed,
        //         &self.settings_interaction,
        //         &self.settings_style,
        //     );

        //     self.g.walk(n_walker, e_walker);
        // }

        self.fit_if_first(&resp, &computed, &mut meta);

        // TODO: create walkers for nodes and edges for drawing and computing state
        // let drawer = Drawer::new(&self.g, &p, &mut meta, &mut computed, &self.settings_style);
        // self.g.walk(|idx: &NodeIndex, n: &Node<N>| {});
        self.draw(&p, &meta, &computed);

        self.handle_node_drag(&resp, &mut computed, &mut meta);
        self.handle_click(&resp, &mut computed, &mut meta);
        self.handle_navigation(ui, &resp, &computed, &mut meta);

        meta.store_into_ui(ui);
        ui.ctx().request_repaint();

        resp
    }
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> GraphView<'a, N, E, Ty> {
    /// Creates a new `GraphView` widget with default navigation and interactions settings.
    /// To customize navigation and interactions use `with_interactions` and `with_navigations` methods.
    pub fn new(g: &'a mut StableGraph<Node<N>, Edge<E>, Ty>) -> Self {
        Self {
            g: GraphWrapper::new(g),

            settings_style: Default::default(),
            settings_interaction: Default::default(),
            setings_navigation: Default::default(),
            changes_sender: Default::default(),
        }
    }

    /// Makes widget interactive sending changes if interaction has occured. Interaction events which
    /// are configured in `settings_interaction` are sent to the channel as soon as the occured.
    pub fn with_interactions(mut self, settings_interaction: &SettingsInteraction) -> Self {
        self.settings_interaction = settings_interaction.clone();
        self
    }

    /// Provide a channel to which changes will be sent.
    pub fn with_changes(mut self, changes_sender: &'a Sender<Change>) -> Self {
        self.changes_sender = Some(changes_sender);
        self
    }

    /// Modifies default behaviour of navigation settings.
    pub fn with_navigations(mut self, settings_navigation: &SettingsNavigation) -> Self {
        self.setings_navigation = settings_navigation.clone();
        self
    }

    /// Modifies default style settings.
    pub fn with_styles(mut self, settings_style: &SettingsStyle) -> Self {
        self.settings_style = settings_style.clone();
        self
    }

    /// Resets navigation metadata
    pub fn reset_metadata(ui: &mut Ui) {
        Metadata::default().store_into_ui(ui);
    }

    /// Gets rect in which graph is contained including node radius
    fn bounding_rect(&self, state: &StateComputed, meta: &mut Metadata) -> Rect {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);

        self.g.nodes_with_context(state).for_each(|(_, n, comp)| {
            let x_minus_rad = n.location().x - comp.radius(meta);
            if x_minus_rad < min_x {
                min_x = x_minus_rad;
            };

            let y_minus_rad = n.location().y - comp.radius(meta);
            if y_minus_rad < min_y {
                min_y = y_minus_rad;
            };

            let x_plus_rad = n.location().x + comp.radius(meta);
            if x_plus_rad > max_x {
                max_x = x_plus_rad;
            };

            let y_plus_rad = n.location().y + comp.radius(meta);
            if y_plus_rad > max_y {
                max_y = y_plus_rad;
            };
        });

        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    }

    /// Fits the graph to the screen if it is the first frame
    fn fit_if_first(&self, r: &Response, comp: &StateComputed, meta: &mut Metadata) {
        if !meta.first_frame {
            return;
        }

        self.fit_to_screen(&r.rect, comp, meta);
        meta.first_frame = false;
    }

    fn handle_click(&mut self, resp: &Response, comp: &mut StateComputed, meta: &mut Metadata) {
        if !resp.clicked() && !resp.double_clicked() {
            return;
        }

        let clickable = self.settings_interaction.node_click
            || self.settings_interaction.node_select
            || self.settings_interaction.node_multiselect
            || self.settings_interaction.node_fold;

        if !(clickable) {
            return;
        }

        // click on empty space
        let node = self.g.node_by_pos(comp, meta, resp.hover_pos().unwrap());
        if node.is_none() {
            let selectable =
                self.settings_interaction.node_select || self.settings_interaction.node_multiselect;
            if selectable {
                self.deselect_all(comp);
            }
            return;
        }

        // first clickf of double click is accepted as single click
        // so if you double click a node it will handle it as click and
        // then double click
        let node_idx = node.unwrap().0;
        if resp.double_clicked() {
            self.handle_node_double_click(node_idx, comp);
            return;
        }
        self.handle_node_click(node_idx, comp);
    }

    fn handle_node_double_click(&mut self, idx: NodeIndex, comp: &mut StateComputed) {
        if !self.settings_interaction.node_click && !self.settings_interaction.node_fold {
            return;
        }

        if self.settings_interaction.node_click {
            self.set_node_double_clicked(idx);
        }

        if !self.settings_interaction.node_fold {
            return;
        }

        if !comp.foldings.is_empty() {
            comp.foldings.roots().iter().for_each(|root_idx| {
                self.set_node_folded(*root_idx, false);
            });
            return;
        }

        self.set_node_folded(idx, true);
    }

    fn handle_node_click(&mut self, idx: NodeIndex, state: &StateComputed) {
        if !self.settings_interaction.node_click && !self.settings_interaction.node_select {
            return;
        }

        if self.settings_interaction.node_click {
            self.set_node_clicked(idx);
        }

        if !self.settings_interaction.node_select {
            return;
        }

        let n = self.g.node(idx).unwrap();
        if n.selected() {
            self.set_node_selected(idx, false);
            return;
        }

        if !self.settings_interaction.node_multiselect {
            self.deselect_all(state);
        }

        self.set_node_selected(idx, true);
    }

    fn handle_node_drag(&mut self, resp: &Response, comp: &mut StateComputed, meta: &mut Metadata) {
        if !self.settings_interaction.node_drag {
            return;
        }

        if resp.drag_started() {
            if let Some((idx, _, _)) = self.g.node_by_pos(comp, meta, resp.hover_pos().unwrap()) {
                self.set_dragged(idx, true);
            }
        }

        if resp.dragged() && comp.dragged.is_some() {
            let n_idx_dragged = comp.dragged.unwrap();
            let delta_in_graph_coords = resp.drag_delta() / meta.zoom;
            self.move_node(n_idx_dragged, delta_in_graph_coords);
        }

        if resp.drag_released() && comp.dragged.is_some() {
            let n_idx = comp.dragged.unwrap();
            self.set_dragged(n_idx, false);
        }
    }

    fn fit_to_screen(&self, rect: &Rect, comp: &StateComputed, meta: &mut Metadata) {
        meta.graph_bounds = self.bounding_rect(comp, meta);

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
        comp: &StateComputed,
        meta: &mut Metadata,
    ) {
        if self.setings_navigation.fit_to_screen {
            return self.fit_to_screen(&resp.rect, comp, meta);
        }

        self.handle_zoom(ui, resp, meta);
        self.handle_pan(resp, comp, meta);
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

    fn handle_pan(&self, resp: &Response, state: &StateComputed, meta: &mut Metadata) {
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

    fn set_node_selected(&mut self, idx: NodeIndex, val: bool) {
        let n = self.g.node_mut(idx).unwrap();
        let change = ChangeNode::change_selected(idx, n.selected(), val);
        n.set_selected(val);
        self.send_changes(Change::node(change));
    }

    fn set_node_folded(&mut self, idx: NodeIndex, val: bool) {
        let n = self.g.node_mut(idx).unwrap();
        let change = ChangeNode::change_folded(idx, n.folded(), val);
        n.set_folded(val);
        self.send_changes(Change::node(change));
    }

    fn set_node_clicked(&mut self, idx: NodeIndex) {
        let change = ChangeNode::clicked(idx);
        self.send_changes(Change::node(change));
    }

    fn set_node_double_clicked(&mut self, idx: NodeIndex) {
        let change = ChangeNode::double_clicked(idx);
        self.send_changes(Change::node(change));
    }

    fn deselect_all(&mut self, state: &StateComputed) {
        if state.selections.is_empty() {
            return;
        }

        // dont need to deselect edges because they are not selectable
        // and subselections are dropped on every frame
        let (selected_nodes, _) = state.selections.elements();

        selected_nodes.iter().for_each(|idx| {
            self.set_node_selected(*idx, false);
        });
    }

    fn set_dragged(&mut self, idx: NodeIndex, val: bool) {
        let n = self.g.node_mut(idx).unwrap();
        let change = ChangeNode::change_dragged(idx, n.dragged(), val);
        n.set_dragged(val);
        self.send_changes(Change::node(change));
    }

    fn move_node(&mut self, idx: NodeIndex, delta: Vec2) {
        let n = self.g.node_mut(idx).unwrap();
        let new_loc = n.location() + delta;
        let change = ChangeNode::change_location(idx, n.location(), new_loc);
        n.set_location(new_loc);
        self.send_changes(Change::node(change));
    }

    fn draw(&self, p: &Painter, meta: &Metadata, comp: &StateComputed) {
        Drawer::new(&self.g, p, meta, comp, &self.settings_style).draw();
    }

    fn send_changes(&self, changes: Change) {
        if let Some(sender) = self.changes_sender {
            sender.send(changes).unwrap();
        }
    }
}
