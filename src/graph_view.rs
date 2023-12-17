use std::marker::PhantomData;

#[cfg(feature = "events")]
use crate::events::{
    Event, PayloadEdgeClick, PayloadEdgeDeselect, PayloadEdgeSelect, PayloadNodeClick,
    PayloadNodeDeselect, PayloadNodeDoubleClick, PayloadNodeDragEnd, PayloadNodeDragStart,
    PayloadNodeMove, PayloadNodeSelect, PayloadPan, PayloadZoom,
};
use crate::{
    draw::Drawer,
    metadata::Metadata,
    settings::SettingsNavigation,
    settings::{SettingsInteraction, SettingsStyle},
    Graph,
};
use crate::{
    draw::{DefaultEdgeShape, DefaultNodeShape, DrawContext},
    DisplayEdge, DisplayNode,
};
#[cfg(feature = "events")]
use crossbeam::channel::Sender;
use egui::{Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
use petgraph::{graph::EdgeIndex, stable_graph::DefaultIx};
use petgraph::{graph::IndexType, Directed};
use petgraph::{stable_graph::NodeIndex, EdgeType};

/// Widget for visualizing and interacting with graphs.
///
/// It implements [egui::Widget] and can be used like any other widget.
///
/// The widget uses a mutable reference to the [petgraph::stable_graph::StableGraph<super::Node<N>, super::Edge<E>>]
/// struct to visualize and interact with the graph. `N` and `E` is arbitrary client data associated with nodes and edges.
/// You can customize the visualization and interaction behavior using [SettingsInteraction], [SettingsNavigation] and [SettingsStyle] structs.
///
/// When any interaction or node property change occurs, the widget sends [Event] struct to the provided
/// [Sender<Event>] channel, which can be set via the `with_interactions` method. The [Event] struct contains information about
/// a change that occurred in the graph. Client can use this information to modify external state of his application if needed.
///
/// When the user performs navigation actions (zoom & pan or fit to screen), they do not
/// produce changes. This is because these actions are performed on the global coordinates and do not change any
/// properties of the nodes or edges.
pub struct GraphView<
    'a,
    N,
    E,
    Ty = Directed,
    Ix = DefaultIx,
    Nd = DefaultNodeShape,
    Ed = DefaultEdgeShape,
> where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
{
    g: &'a mut Graph<N, E, Ty, Ix, Nd, Ed>,

    settings_interaction: SettingsInteraction,
    settings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,

    #[cfg(feature = "events")]
    events_publisher: Option<&'a Sender<Event>>,

    _marker: PhantomData<(Nd, Ed)>,
}

impl<'a, N, E, Ty, Ix, Nd, Ed> Widget for &mut GraphView<'a, N, E, Ty, Ix, Nd, Ed>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
{
    fn ui(self, ui: &mut Ui) -> Response {
        let (resp, p) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        let mut meta = Metadata::get(ui);
        self.sync_state(&mut meta);

        self.handle_fit_to_screen(&resp, &mut meta);
        self.handle_navigation(ui, &resp, &mut meta);

        self.handle_node_drag(&resp, &mut meta);
        self.handle_click(&resp, &mut meta);

        let is_directed = self.g.is_directed();
        Drawer::<N, E, Ty, Ix, Nd, Ed>::new(
            p,
            self.g,
            &DrawContext {
                ctx: ui.ctx(),
                meta: &meta,
                is_directed,
                style: &self.settings_style,
            },
        )
        .draw();

        meta.first_frame = false;
        meta.store_into_ui(ui);
        ui.ctx().request_repaint();

        resp
    }
}

impl<'a, N, E, Ty, Ix, Dn, De> GraphView<'a, N, E, Ty, Ix, Dn, De>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    /// Creates a new `GraphView` widget with default navigation and interactions settings.
    /// To customize navigation and interactions use `with_interactions` and `with_navigations` methods.
    pub fn new(g: &'a mut Graph<N, E, Ty, Ix, Dn, De>) -> Self {
        Self {
            g,

            settings_style: Default::default(),
            settings_interaction: Default::default(),
            settings_navigation: Default::default(),

            #[cfg(feature = "events")]
            events_publisher: Default::default(),

            _marker: PhantomData,
        }
    }

    /// Makes widget interactive according to the provided settings.
    pub fn with_interactions(mut self, settings_interaction: &SettingsInteraction) -> Self {
        self.settings_interaction = settings_interaction.clone();
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

    #[cfg(feature = "events")]
    /// Allows to supply channel where events happening in the graph will be reported.
    pub fn with_events(mut self, events_publisher: &'a Sender<Event>) -> Self {
        self.events_publisher = Some(events_publisher);
        self
    }

    fn sync_state(&mut self, meta: &mut Metadata) {
        let mut selected_nodes = Vec::new();
        let mut selected_edges = Vec::new();
        let mut dragged = None;

        meta.reset_bounds_iterator();
        self.g.nodes_iter().for_each(|(idx, n)| {
            if n.dragged() {
                dragged = Some(idx);
            }
            if n.selected() {
                selected_nodes.push(idx);
            }

            meta.comp_iter_bounds(n);
        });

        self.g.edges_iter().for_each(|(idx, e)| {
            if e.selected() {
                selected_edges.push(idx);
            }
        });

        self.g.set_selected_nodes(selected_nodes);
        self.g.set_selected_edges(selected_edges);
        self.g.set_dragged_node(dragged);
    }

    /// Fits the graph to the screen if it is the first frame or
    /// fit to screen setting is enabled;
    fn handle_fit_to_screen(&self, r: &Response, meta: &mut Metadata) {
        if !meta.first_frame && !self.settings_navigation.fit_to_screen_enabled {
            return;
        }

        self.fit_to_screen(&r.rect, meta);
    }

    fn handle_click(&mut self, resp: &Response, meta: &mut Metadata) {
        if !resp.clicked() && !resp.double_clicked() {
            return;
        }

        let clickable = self.settings_interaction.node_clicking_enabled
            || self.settings_interaction.node_selection_enabled
            || self.settings_interaction.node_selection_multi_enabled
            || self.settings_interaction.edge_clicking_enabled
            || self.settings_interaction.edge_selection_enabled
            || self.settings_interaction.edge_selection_multi_enabled;

        if !(clickable) {
            return;
        }

        let found_edge = self.g.edge_by_screen_pos(meta, resp.hover_pos().unwrap());
        let found_node = self.g.node_by_screen_pos(meta, resp.hover_pos().unwrap());
        if found_node.is_none() && found_edge.is_none() {
            // click on empty space
            let nodes_selectable = self.settings_interaction.node_selection_enabled
                || self.settings_interaction.node_selection_multi_enabled;
            if nodes_selectable {
                self.deselect_all_nodes();
            }

            let edges_selectable = self.settings_interaction.edge_selection_enabled
                || self.settings_interaction.edge_selection_multi_enabled;
            if edges_selectable {
                self.deselect_all_edges();
            }
            return;
        }

        if let Some(idx) = found_node {
            // first click of double click is handled by the lib as single click
            // so if you double click a node it will handle it as single click at first
            // and only after as double click
            if resp.double_clicked() {
                self.handle_node_double_click(idx);
                return;
            }
            self.handle_node_click(idx);
            return;
        }

        if let Some(edge_idx) = found_edge {
            self.handle_edge_click(edge_idx);
        }
    }

    fn handle_node_double_click(&mut self, idx: NodeIndex<Ix>) {
        if !self.settings_interaction.node_clicking_enabled {
            return;
        }

        if self.settings_interaction.node_clicking_enabled {
            self.set_node_double_clicked(idx);
        }
    }

    fn handle_node_click(&mut self, idx: NodeIndex<Ix>) {
        if !self.settings_interaction.node_clicking_enabled
            && !self.settings_interaction.node_selection_enabled
        {
            return;
        }

        if self.settings_interaction.node_clicking_enabled {
            self.set_node_clicked(idx);
        }

        if !self.settings_interaction.node_selection_enabled {
            return;
        }

        let n = self.g.node(idx).unwrap();
        if n.selected() {
            self.deselect_node(idx);
            return;
        }

        if !self.settings_interaction.node_selection_multi_enabled {
            self.deselect_all();
        }

        self.select_node(idx);
    }

    fn handle_edge_click(&mut self, idx: EdgeIndex<Ix>) {
        if !self.settings_interaction.edge_clicking_enabled
            && !self.settings_interaction.edge_selection_enabled
        {
            return;
        }

        if self.settings_interaction.edge_clicking_enabled {
            self.set_edge_clicked(idx);
        }

        if !self.settings_interaction.edge_selection_enabled {
            return;
        }

        let e = self.g.edge(idx).unwrap();
        if e.selected() {
            self.deselect_edge(idx);
            return;
        }

        if !self.settings_interaction.edge_selection_multi_enabled {
            self.deselect_all();
        }

        self.select_edge(idx);
    }

    fn handle_node_drag(&mut self, resp: &Response, meta: &mut Metadata) {
        if !self.settings_interaction.dragging_enabled {
            return;
        }

        if resp.drag_started() {
            if let Some(idx) = self.g.node_by_screen_pos(meta, resp.hover_pos().unwrap()) {
                self.set_drag_start(idx);
            }
        }

        if resp.dragged()
            && self.g.dragged_node().is_some()
            && (resp.drag_delta().x.abs() > 0. || resp.drag_delta().y.abs() > 0.)
        {
            let n_idx_dragged = self.g.dragged_node().unwrap();
            let delta_in_graph_coords = resp.drag_delta() / meta.zoom;
            self.move_node(n_idx_dragged, delta_in_graph_coords);
        }

        if resp.drag_released() && self.g.dragged_node().is_some() {
            let n_idx = self.g.dragged_node().unwrap();
            self.set_drag_end(n_idx);
        }
    }

    fn fit_to_screen(&self, rect: &Rect, meta: &mut Metadata) {
        // calculate graph dimensions with decorative padding
        let bounds = meta.graph_bounds();
        let mut diag = bounds.max - bounds.min;

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
        let graph_center = (bounds.min.to_vec2() + bounds.max.to_vec2()) / 2.0;

        // adjust the pan value to align the centers of the graph and the canvas
        let new_pan = rect.center().to_vec2() - graph_center * new_zoom;
        self.set_pan(new_pan, meta);
    }

    fn handle_navigation(&self, ui: &Ui, resp: &Response, meta: &mut Metadata) {
        if !meta.first_frame {
            meta.pan += resp.rect.left_top() - meta.top_left;
        }
        meta.top_left = resp.rect.left_top();

        self.handle_zoom(ui, resp, meta);
        self.handle_pan(resp, meta);
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

            let step = self.settings_navigation.zoom_speed * (delta - 1.).signum();
            self.zoom(&resp.rect, step, i.pointer.hover_pos(), meta);
        });
    }

    fn handle_pan(&self, resp: &Response, meta: &mut Metadata) {
        if !self.settings_navigation.zoom_and_pan_enabled {
            return;
        }

        if resp.dragged()
            && self.g.dragged_node().is_none()
            && (resp.drag_delta().x.abs() > 0. || resp.drag_delta().y.abs() > 0.)
        {
            let new_pan = meta.pan + resp.drag_delta();
            self.set_pan(new_pan, meta);
        }
    }

    /// Zooms the graph by the given delta. It also compensates with pan to keep the zoom center in the same place.
    fn zoom(&self, rect: &Rect, delta: f32, zoom_center: Option<Pos2>, meta: &mut Metadata) {
        let center_pos = zoom_center.unwrap_or(rect.center()).to_vec2();
        let graph_center_pos = (center_pos - meta.pan) / meta.zoom;
        let factor = 1. + delta;
        let new_zoom = meta.zoom * factor;

        let pan_delta = graph_center_pos * meta.zoom - graph_center_pos * new_zoom;
        let new_pan = meta.pan + pan_delta;

        self.set_pan(new_pan, meta);
        self.set_zoom(new_zoom, meta);
    }

    fn select_node(&mut self, idx: NodeIndex<Ix>) {
        let n = self.g.node_mut(idx).unwrap();
        n.set_selected(true);

        #[cfg(feature = "events")]
        self.publish_event(Event::NodeSelect(PayloadNodeSelect { id: idx.index() }));
    }

    fn deselect_node(&mut self, idx: NodeIndex<Ix>) {
        let n = self.g.node_mut(idx).unwrap();
        n.set_selected(false);

        #[cfg(feature = "events")]
        self.publish_event(Event::NodeDeselect(PayloadNodeDeselect { id: idx.index() }));
    }

    #[allow(unused_variables)]
    fn set_node_clicked(&mut self, idx: NodeIndex<Ix>) {
        #[cfg(feature = "events")]
        self.publish_event(Event::NodeClick(PayloadNodeClick { id: idx.index() }));
    }

    #[allow(unused_variables)]
    fn set_node_double_clicked(&mut self, idx: NodeIndex<Ix>) {
        #[cfg(feature = "events")]
        self.publish_event(Event::NodeDoubleClick(PayloadNodeDoubleClick {
            id: idx.index(),
        }));
    }

    #[allow(unused_variables)]
    fn set_edge_clicked(&mut self, idx: EdgeIndex<Ix>) {
        #[cfg(feature = "events")]
        self.publish_event(Event::EdgeClick(PayloadEdgeClick { id: idx.index() }));
    }

    fn select_edge(&mut self, idx: EdgeIndex<Ix>) {
        let e = self.g.edge_mut(idx).unwrap();
        e.set_selected(true);

        #[cfg(feature = "events")]
        self.publish_event(Event::EdgeSelect(PayloadEdgeSelect { id: idx.index() }));
    }

    fn deselect_edge(&mut self, idx: EdgeIndex<Ix>) {
        let e = self.g.edge_mut(idx).unwrap();
        e.set_selected(false);

        #[cfg(feature = "events")]
        self.publish_event(Event::EdgeDeselect(PayloadEdgeDeselect { id: idx.index() }));
    }

    /// Deselects all nodes AND edges.
    fn deselect_all(&mut self) {
        self.deselect_all_nodes();
        self.deselect_all_edges();
    }

    fn deselect_all_nodes(&mut self) {
        let selected_nodes = self.g.selected_nodes().to_vec();
        selected_nodes.into_iter().for_each(|idx| {
            self.deselect_node(idx);
        });
    }

    fn deselect_all_edges(&mut self) {
        let selected_edges = self.g.selected_edges().to_vec();
        selected_edges.into_iter().for_each(|idx| {
            self.deselect_edge(idx);
        });
    }

    fn move_node(&mut self, idx: NodeIndex<Ix>, delta: Vec2) {
        let n = self.g.node_mut(idx).unwrap();
        let new_loc = n.location() + delta;
        n.set_location(new_loc);

        #[cfg(feature = "events")]
        self.publish_event(Event::NodeMove(PayloadNodeMove {
            id: idx.index(),
            diff: delta.into(),
            new_pos: [new_loc.x, new_loc.y],
        }));
    }

    fn set_drag_start(&mut self, idx: NodeIndex<Ix>) {
        let n = self.g.node_mut(idx).unwrap();
        n.set_dragged(true);

        #[cfg(feature = "events")]
        self.publish_event(Event::NodeDragStart(PayloadNodeDragStart {
            id: idx.index(),
        }));
    }

    fn set_drag_end(&mut self, idx: NodeIndex<Ix>) {
        let n = self.g.node_mut(idx).unwrap();
        n.set_dragged(false);

        #[cfg(feature = "events")]
        self.publish_event(Event::NodeDragEnd(PayloadNodeDragEnd { id: idx.index() }));
    }

    #[allow(unused_variables)]
    fn set_pan(&self, new_pan: Vec2, meta: &mut Metadata) {
        let diff = new_pan - meta.pan;
        meta.pan = new_pan;

        #[cfg(feature = "events")]
        self.publish_event(Event::Pan(PayloadPan {
            diff: diff.into(),
            new_pan: new_pan.into(),
        }));
    }

    #[allow(unused_variables)]
    fn set_zoom(&self, new_zoom: f32, meta: &mut Metadata) {
        let diff = new_zoom - meta.zoom;
        meta.zoom = new_zoom;

        #[cfg(feature = "events")]
        self.publish_event(Event::Zoom(PayloadZoom { diff, new_zoom }));
    }

    #[cfg(feature = "events")]
    fn publish_event(&self, event: Event) {
        if let Some(sender) = self.events_publisher {
            sender.send(event).unwrap();
        }
    }
}
