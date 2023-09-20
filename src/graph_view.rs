use crossbeam::channel::Sender;
use egui::{Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
use petgraph::{
    stable_graph::NodeIndex,
    visit::{IntoNodeIdentifiers, IntoNodeReferences},
    EdgeType,
};

use crate::{
    change::{Change, ChangeNode, ChangeSubgraph},
    metadata::Metadata,
    settings::SettingsNavigation,
    settings::{SettingsInteraction, SettingsStyle},
    state_computed::StateComputed,
    Drawer, Graph,
};

/// Widget for visualizing and interacting with graphs.
///
/// It implements [egui::Widget] and can be used like any other widget.
///
/// The widget uses a mutable reference to the [StableGraph<egui_graphs::Node<N>, egui_graphs::Edge<E>>]
/// struct to visualize and interact with the graph. `N` and `E` is arbitrary client data associated with nodes and edges.
/// You can customize the visualization and interaction behavior using [SettingsInteraction], [SettingsNavigation] and [SettingsStyle] structs.
///
/// When any interaction or node property change occurs, the widget sends [Change] struct to the provided
/// [Sender<Change>] channel, which can be set via the `with_interactions` method. The [Change] struct contains information about
/// a change that occurred in the graph. Client can use this information to modify external state of his application if needed.
///
/// When the user performs navigation actions (zoom & pan, fit to screen), they do not
/// produce changes. This is because these actions are performed on the global coordinates and do not change any
/// properties of the nodes or edges.
pub struct GraphView<'a, N: Clone, E: Clone, Ty: EdgeType> {
    settings_interaction: SettingsInteraction,
    settings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,
    g: &'a mut Graph<N, E, Ty>,
    changes_sender: Option<&'a Sender<Change>>,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Widget for &mut GraphView<'a, N, E, Ty> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (resp, p) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        let mut meta = Metadata::get(ui);
        let mut computed = self.compute_computed(&meta);

        self.handle_fit_to_screen(&resp, &mut meta, &computed);
        self.handle_navigation(ui, &resp, &mut meta, &computed);

        self.handle_node_drag(&resp, &mut computed, &mut meta);
        self.handle_click(&resp, &mut computed, &mut meta);

        Drawer::new(p, self.g, &meta, &computed, &self.settings_style).draw();

        meta.store_into_ui(ui);
        ui.ctx().request_repaint();

        resp
    }
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> GraphView<'a, N, E, Ty> {
    /// Creates a new `GraphView` widget with default navigation and interactions settings.
    /// To customize navigation and interactions use `with_interactions` and `with_navigations` methods.
    pub fn new(g: &'a mut Graph<N, E, Ty>) -> Self {
        Self {
            g,

            settings_style: Default::default(),
            settings_interaction: Default::default(),
            settings_navigation: Default::default(),
            changes_sender: Default::default(),
        }
    }

    /// Makes widget interactive according to the provided settings.
    pub fn with_interactions(mut self, settings_interaction: &SettingsInteraction) -> Self {
        self.settings_interaction = settings_interaction.clone();
        self
    }

    /// Make every interaction send [`Change`] to the provided [`crossbeam::channel::Sender`] as soon as interaction happens.
    ///
    /// Change events can be used to handle interactions on the application side.
    pub fn with_changes(mut self, changes_sender: &'a Sender<Change>) -> Self {
        self.changes_sender = Some(changes_sender);
        self
    }

    /// Modifies default behaviour of navigation settings.
    pub fn with_navigations(mut self, settings_navigation: &SettingsNavigation) -> Self {
        self.settings_navigation = settings_navigation.clone();
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

    fn compute_computed(&mut self, meta: &Metadata) -> StateComputed {
        let mut computed = StateComputed::default();
        let idxs = self.g.g.node_indices().collect::<Vec<_>>();
        idxs.iter().for_each(|idx| {
            let new_radius = computed.compute_for_node(
                self.g,
                *idx,
                meta,
                &self.settings_interaction,
                &self.settings_style,
            );
            let n = self.g.node_mut(*idx).unwrap().set_radius(new_radius);
        });

        self.g
            .edges_iter()
            .for_each(|(idx, e)| computed.compute_for_edge(idx, e, meta));

        computed.compute_graph_bounds();

        computed
    }

    /// Fits the graph to the screen if it is the first frame or
    /// fit to screen setting is enabled;
    fn handle_fit_to_screen(&self, r: &Response, meta: &mut Metadata, comp: &StateComputed) {
        if !meta.first_frame && !self.settings_navigation.fit_to_screen_enabled {
            return;
        }

        self.fit_to_screen(&r.rect, meta, comp);
        meta.first_frame = false;
    }

    fn handle_click(&mut self, resp: &Response, comp: &mut StateComputed, meta: &mut Metadata) {
        if !resp.clicked() && !resp.double_clicked() {
            return;
        }

        let clickable = self.settings_interaction.clicking_enabled
            || self.settings_interaction.selection_enabled
            || self.settings_interaction.selection_multi_enabled
            || self.settings_interaction.folding_enabled;

        if !(clickable) {
            return;
        }

        let node = self.g.node_by_pos(meta, resp.hover_pos().unwrap());
        if node.is_none() {
            // click on empty space
            let selectable = self.settings_interaction.selection_enabled
                || self.settings_interaction.selection_multi_enabled;
            if selectable {
                self.deselect_all(comp);
            }
            return;
        }

        // first click of double click is handleed by the lib as single click
        // so if you double click a node it will handle it as single click at first
        // and only after as double click
        let node_idx = node.unwrap().0;
        if resp.double_clicked() {
            self.handle_node_double_click(node_idx, comp);
            return;
        }
        self.handle_node_click(node_idx, comp);
    }

    fn handle_node_double_click(&mut self, idx: NodeIndex, comp: &mut StateComputed) {
        if !self.settings_interaction.clicking_enabled && !self.settings_interaction.folding_enabled
        {
            return;
        }

        if self.settings_interaction.clicking_enabled {
            self.set_node_double_clicked(idx);
        }

        if !self.settings_interaction.folding_enabled {
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

    fn handle_node_click(&mut self, idx: NodeIndex, comp: &mut StateComputed) {
        if !self.settings_interaction.clicking_enabled
            && !self.settings_interaction.selection_enabled
        {
            return;
        }

        if self.settings_interaction.clicking_enabled {
            self.set_node_clicked(idx);
        }

        if !self.settings_interaction.selection_enabled {
            return;
        }

        let n = self.g.node(idx).unwrap();
        if n.selected() {
            self.select_node(idx);
            return;
        }

        if !self.settings_interaction.selection_multi_enabled {
            self.deselect_all(comp);
        }

        self.select_node(idx);
    }

    fn handle_node_drag(&mut self, resp: &Response, comp: &mut StateComputed, meta: &mut Metadata) {
        if !self.settings_interaction.dragging_enabled {
            return;
        }

        if resp.drag_started() {
            if let Some((idx, _)) = self.g.node_by_pos(meta, resp.hover_pos().unwrap()) {
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

    fn fit_to_screen(&self, rect: &Rect, meta: &mut Metadata, comp: &StateComputed) {
        // calculate graph dimensions with decorative padding
        let mut diag = comp.graph_bounds.max - comp.graph_bounds.min;

        // if the graph is empty or consists from one node, use a default size
        if diag == Vec2::ZERO {
            diag = Vec2::new(1., 100.);
        }

        let graph_size = diag * (1. + self.settings_navigation.screen_padding);
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
            (comp.graph_bounds.min.to_vec2() + comp.graph_bounds.max.to_vec2()) / 2.0;

        // adjust the pan value to align the centers of the graph and the canvas
        meta.pan = rect.center().to_vec2() - graph_center * new_zoom;
    }

    fn handle_navigation(
        &self,
        ui: &Ui,
        resp: &Response,
        meta: &mut Metadata,
        comp: &StateComputed,
    ) {
        self.handle_zoom(ui, resp, meta);
        self.handle_pan(resp, meta, comp);
    }

    fn handle_zoom(&self, ui: &Ui, resp: &Response, meta: &mut Metadata) {
        if !self.settings_navigation.zoom_and_pan_enabled {
            return;
        }

        ui.input(|i| {
            let delta = i.zoom_delta();
            if delta == 1. {
                return;
            }

            let step = self.settings_navigation.zoom_speed * (1. - delta).signum();
            self.zoom(&resp.rect, step, i.pointer.hover_pos(), meta);
        });
    }

    fn handle_pan(&self, resp: &Response, meta: &mut Metadata, comp: &StateComputed) {
        if !self.settings_navigation.zoom_and_pan_enabled {
            return;
        }

        if resp.dragged() && comp.dragged.is_none() {
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

    fn deselect_node(&mut self, idx: NodeIndex) {
        let n = self.g.node_mut(idx).unwrap();
        let change = ChangeNode::change_selected(idx, n.selected(), false);
        n.set_selected(false);

        self.send_changes(Change::node(change));
    }

    fn select_node(&mut self, idx: NodeIndex) {
        let c = {
            let n = self.g.node_mut(idx).unwrap();
            n.set_selected(true);
            ChangeNode::change_selected(idx, n.selected(), true)
        };
        self.send_changes(Change::node(c));
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

    fn deselect_all(&mut self, comp: &mut StateComputed) {
        if comp.selections.is_empty() {
            return;
        }

        let (subselected, _) = comp.selections.elements();
        subselected.iter().for_each(|idx| self.deselect_node(*idx));
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

    fn send_selected(&self, comp: &StateComputed) {
        if comp.selections.is_empty() {
            return;
        }

        comp.selections.subgraphs().for_each(|(root, subgraph)| {
            self.send_changes(Change::SubGraph(ChangeSubgraph::change_selected(
                *root,
                subgraph.clone(),
            )))
        })
    }

    fn send_folded(&self, comp: &StateComputed) {
        if comp.foldings.is_empty() {
            return;
        }

        comp.foldings.subgraphs().for_each(|(root, subgraph)| {
            self.send_changes(Change::SubGraph(ChangeSubgraph::change_folded(
                *root,
                subgraph.clone(),
            )))
        })
    }

    fn send_changes(&self, changes: Change) {
        if let Some(sender) = self.changes_sender {
            sender.send(changes).unwrap();
        }
    }
}
