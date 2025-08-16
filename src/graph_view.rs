use std::marker::PhantomData;

use crate::{
    draw::{drawer::Drawer, DefaultEdgeShape, DefaultNodeShape, DrawContext},
    layouts::{self, Layout, LayoutState},
    metadata::Metadata,
    settings::{SettingsInteraction, SettingsNavigation, SettingsStyle},
    DisplayEdge, DisplayNode, Graph,
};

use egui::{Id, PointerButton, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
use instant::Instant;

use petgraph::{graph::EdgeIndex, stable_graph::DefaultIx};
use petgraph::{graph::IndexType, Directed};
use petgraph::{stable_graph::NodeIndex, EdgeType};

const KEY_LAYOUT: &str = "egui_graphs_layout";

// Shared cores to avoid duplication across general and force-run variants.
fn ff_steps_core<N, E, Ty, Ix, Dn, De, S, L, Pre, Post>(
    ui: &egui::Ui,
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    target_steps: u32,
    budget_millis: Option<u64>,
    pre_toggle: Pre,
    post_toggle: Post,
) -> u32
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
    S: LayoutState,
    L: Layout<S>,
    Pre: Fn(&mut S) -> Option<bool>,
    Post: Fn(&mut S, Option<bool>),
{
    if target_steps == 0 || g.node_count() == 0 {
        return 0;
    }
    let mut state = GraphView::<N, E, Ty, Ix, Dn, De, S, L>::get_layout_state(ui);
    let token = pre_toggle(&mut state);
    let mut layout = L::from_state(state);
    let start = Instant::now();
    let mut done = 0u32;
    while done < target_steps {
        if let Some(ms) = budget_millis {
            if start.elapsed().as_millis() as u64 >= ms {
                break;
            }
        }
        layout.next(g, ui);
        done += 1;
    }
    let mut new_state = layout.state();
    post_toggle(&mut new_state, token);
    GraphView::<N, E, Ty, Ix, Dn, De, S, L>::set_layout_state(ui, new_state);
    done
}

#[allow(clippy::too_many_arguments)]
fn ff_until_stable_core<N, E, Ty, Ix, Dn, De, S, L, Metric, Pre, Post>(
    ui: &egui::Ui,
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    epsilon: f32,
    max_steps: u32,
    budget_millis: Option<u64>,
    metric: Metric,
    pre_toggle: Pre,
    post_toggle: Post,
) -> (u32, f32)
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
    S: LayoutState,
    L: Layout<S>,
    Metric: Fn(&S) -> Option<f32>,
    Pre: Fn(&mut S) -> Option<bool>,
    Post: Fn(&mut S, Option<bool>),
{
    if g.node_count() == 0 || max_steps == 0 {
        return (0, 0.0);
    }

    let mut state = GraphView::<N, E, Ty, Ix, Dn, De, S, L>::get_layout_state(ui);
    let token = pre_toggle(&mut state);
    let mut layout = L::from_state(state);

    let start = Instant::now();
    let mut steps_done = 0u32;
    let mut last_avg = f32::INFINITY;
    let indices: Vec<_> = g.g().node_indices().collect();
    let mut prev_positions = Vec::with_capacity(indices.len());
    prev_positions.extend(
        indices
            .iter()
            .map(|&idx| g.g().node_weight(idx).unwrap().location()),
    );

    while steps_done < max_steps {
        if let Some(ms) = budget_millis {
            if start.elapsed().as_millis() as u64 >= ms {
                break;
            }
        }
        layout.next(g, ui);
        steps_done += 1;

        if let Some(avg) = metric(&layout.state()) {
            last_avg = avg;
        } else {
            let mut sum = 0.0f32;
            let mut count = 0usize;
            for (i, &idx) in indices.iter().enumerate() {
                if let Some(n) = g.g().node_weight(idx) {
                    let cur = n.location();
                    let d = (cur - prev_positions[i]).length();
                    sum += d;
                    count += 1;
                    prev_positions[i] = cur;
                }
            }
            last_avg = if count == 0 { 0.0 } else { sum / count as f32 };
        }

        if last_avg < epsilon {
            break;
        }
    }

    let mut new_state = layout.state();
    post_toggle(&mut new_state, token);
    GraphView::<N, E, Ty, Ix, Dn, De, S, L>::set_layout_state(ui, new_state);
    (
        steps_done,
        if last_avg.is_finite() { last_avg } else { 0.0 },
    )
}

pub type DefaultGraphView<'a> = GraphView<
    'a,
    (),
    (),
    Directed,
    DefaultIx,
    DefaultNodeShape,
    DefaultEdgeShape,
    layouts::random::State,
    layouts::random::Random,
>;

#[cfg(feature = "events")]
use crate::events::{
    Event, PayloadEdgeClick, PayloadEdgeDeselect, PayloadEdgeSelect, PayloadNodeClick,
    PayloadNodeDeselect, PayloadNodeDoubleClick, PayloadNodeDragEnd, PayloadNodeDragStart,
    PayloadNodeHoverEnter, PayloadNodeHoverLeave, PayloadNodeMove, PayloadNodeSelect, PayloadPan,
    PayloadZoom,
};
#[cfg(feature = "events")]
use crossbeam::channel::Sender;

// Effective interaction flags after applying master->child rules.
#[derive(Clone, Copy, Debug, Default)]
struct EffectiveInteraction {
    dragging: bool,
    hover: bool,
    node_clicking: bool,
    node_selection: bool,
    node_selection_multi: bool,
    edge_clicking: bool,
    edge_selection: bool,
    edge_selection_multi: bool,
}

/// Widget for visualizing and interacting with graphs.
///
/// It implements [`egui::Widget`] and can be used like any other widget.
///
/// The widget uses a mutable reference to the [`petgraph::stable_graph::StableGraph`<`super::Node`<N>, `super::Edge`<E>>]
/// struct to visualize and interact with the graph. `N` and `E` is arbitrary client data associated with nodes and edges.
/// You can customize the visualization and interaction behavior using [`SettingsInteraction`], [`SettingsNavigation`] and [`SettingsStyle`] structs.
///
/// When any interaction or node property change occurs, the widget sends [`Event`] struct to the provided
/// [`Sender<Event>`] channel, which can be set via the `with_interactions` method. The [`Event`] struct contains information about
/// a change that occurred in the graph. Client can use this information to modify external state of his application if needed.
///
/// When the user performs navigation actions (zoom & pan or fit to screen), they do not
/// produce changes. This is because these actions are performed on the global coordinates and do not change any
/// properties of the nodes or edges.
pub struct GraphView<
    'a,
    N = (),
    E = (),
    Ty = Directed,
    Ix = DefaultIx,
    Nd = DefaultNodeShape,
    Ed = DefaultEdgeShape,
    S = layouts::random::State,
    L = layouts::random::Random,
> where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
    S: LayoutState,
    L: Layout<S>,
{
    g: &'a mut Graph<N, E, Ty, Ix, Nd, Ed>,

    settings_interaction: SettingsInteraction,
    settings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,

    #[cfg(feature = "events")]
    events_publisher: Option<&'a Sender<Event>>,

    _marker: PhantomData<(Nd, Ed, L, S)>,
}

impl<N, E, Ty, Ix, Nd, Ed, S, L> Widget for &mut GraphView<'_, N, E, Ty, Ix, Nd, Ed, S, L>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Nd: DisplayNode<N, E, Ty, Ix>,
    Ed: DisplayEdge<N, E, Ty, Ix, Nd>,
    S: LayoutState,
    L: Layout<S>,
{
    fn ui(self, ui: &mut Ui) -> Response {
        self.sync_layout(ui);

        let mut meta = Metadata::load(ui);
        self.sync_state(&mut meta);

        // Compute effective interactions once per frame
        let eff = self.effective();

        let (resp, p) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        // Hover detection and cursor update happens as early as possible using current input state
        self.handle_hover(ui, &resp, &mut meta, eff);
        self.handle_fit_to_screen(&resp, &mut meta);
        // Handle node drag before navigation so pan doesn't kick in on the first frame
        // when starting a node drag.
        self.handle_node_drag(&resp, &mut meta, eff);
        self.handle_navigation(ui, &resp, &mut meta, eff);
        self.handle_click(&resp, &mut meta, eff);

        Drawer::<N, E, Ty, Ix, Nd, Ed, S, L>::new(
            self.g,
            &DrawContext {
                ctx: ui.ctx(),
                painter: &p,
                meta: &meta,
                is_directed: self.g.is_directed(),
                style: &self.settings_style,
            },
        )
        .draw();

        meta.first_frame = false;
        meta.save(ui);

        ui.ctx().request_repaint();

        resp
    }
}

// Constructor and lifetime-bound methods
impl<'a, N, E, Ty, Ix, Dn, De, S, L> GraphView<'a, N, E, Ty, Ix, Dn, De, S, L>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
    S: LayoutState,
    L: Layout<S>,
{
    /// Creates a new `GraphView` widget with default navigation and interactions settings.
    /// To customize navigation and interactions use `with_interactions` and `with_navigations` methods.
    pub fn new(g: &'a mut Graph<N, E, Ty, Ix, Dn, De>) -> Self {
        Self {
            g,

            settings_style: SettingsStyle::default(),
            settings_interaction: SettingsInteraction::default(),
            settings_navigation: SettingsNavigation::default(),

            #[cfg(feature = "events")]
            events_publisher: Option::default(),

            _marker: PhantomData,
        }
    }

    #[cfg(feature = "events")]
    /// Allows to supply channel where events happening in the graph will be reported.
    pub fn with_events(mut self, events_publisher: &'a Sender<Event>) -> Self {
        self.events_publisher = Some(events_publisher);
        self
    }
}

impl<N, E, Ty, Ix, Dn, De, S, L> GraphView<'_, N, E, Ty, Ix, Dn, De, S, L>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
    S: LayoutState,
    L: Layout<S>,
{
    /// Compute effective interactions, honoring master->child rules described in docs:
    /// - Dragging enabled implies node click + hover are enabled.
    /// - Selection enabled (node/edge) implies node click + hover enabled.
    /// - Multi-selection enabled (node/edge) implies node click + hover + selection enabled.
    fn effective(&self) -> EffectiveInteraction {
        let si = &self.settings_interaction;

        let mut eff = EffectiveInteraction {
            dragging: si.dragging_enabled,
            hover: si.hover_enabled,
            node_clicking: si.node_clicking_enabled,
            node_selection: si.node_selection_enabled,
            node_selection_multi: si.node_selection_multi_enabled,
            edge_clicking: si.edge_clicking_enabled,
            edge_selection: si.edge_selection_enabled,
            edge_selection_multi: si.edge_selection_multi_enabled,
        };

        // Master: dragging -> children
        if eff.dragging {
            eff.node_clicking = true;
            eff.hover = true;
        }
        // Master: node selection -> children
        if eff.node_selection {
            eff.node_clicking = true;
            eff.hover = true;
        }
        // Master: edge selection -> children
        if eff.edge_selection {
            eff.node_clicking = true;
            eff.hover = true;
        }
        // Master: node multiselection -> children
        if eff.node_selection_multi {
            eff.node_selection = true;
            eff.node_clicking = true;
            eff.hover = true;
        }
        // Master: edge multiselection -> children
        if eff.edge_selection_multi {
            eff.edge_selection = true;
            eff.node_clicking = true;
            eff.hover = true;
        }

        eff
    }

    fn handle_hover(
        &mut self,
        ui: &Ui,
        resp: &Response,
        meta: &mut Metadata,
        eff: EffectiveInteraction,
    ) {
        if self.g.dragged_node().is_some() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }

        if !eff.hover {
            return;
        }

        let hovered_now = if let Some(pos) = resp.hover_pos() {
            self.g.node_by_screen_pos(meta, pos)
        } else {
            None
        };

        if hovered_now.is_some() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }

        let prev = self.g.hovered_node();
        if hovered_now != prev {
            if let Some(prev_idx) = prev {
                #[cfg(feature = "events")]
                {
                    self.publish_event(Event::NodeHoverLeave(PayloadNodeHoverLeave {
                        id: prev_idx.index(),
                    }));
                }
                #[cfg(not(feature = "events"))]
                {
                    let _ = prev_idx;
                }
                if let Some(n) = self.g.node_mut(prev_idx) {
                    n.set_hovered(false);
                }
            }
            if let Some(cur_idx) = hovered_now {
                #[cfg(feature = "events")]
                {
                    self.publish_event(Event::NodeHoverEnter(PayloadNodeHoverEnter {
                        id: cur_idx.index(),
                    }));
                }
                #[cfg(not(feature = "events"))]
                {
                    let _ = cur_idx;
                }
                if let Some(n) = self.g.node_mut(cur_idx) {
                    n.set_hovered(true);
                }
            }
            self.g.set_hovered_node(hovered_now);
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

    /// Helper to reset both [`Metadata`] and [`Layout`] cache. Can be useful when you want to change layout
    /// in runtime
    pub fn reset(ui: &mut Ui) {
        GraphView::<N, E, Ty, Ix, Dn, De, S, L>::reset_metadata(ui);
        GraphView::<N, E, Ty, Ix, Dn, De, S, L>::reset_layout(ui);
    }

    /// Resets [`Metadata`] state
    pub fn reset_metadata(ui: &mut Ui) {
        Metadata::default().save(ui);
    }

    /// Resets [`Layout`] state
    pub fn reset_layout(ui: &mut Ui) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::new(KEY_LAYOUT), S::default());
        });
    }

    /// Loads current persisted layout state (or default if none). Useful for external UI panels.
    pub fn get_layout_state(ui: &egui::Ui) -> S {
        ui.data_mut(|data| {
            data.get_persisted::<S>(Id::new(KEY_LAYOUT))
                .unwrap_or_default()
        })
    }

    /// Persists a new layout state so that on the next frame it will be applied.
    pub fn set_layout_state(ui: &egui::Ui, state: S) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::new(KEY_LAYOUT), state);
        });
    }

    /// Advance the active layout simulation by a fixed number of steps immediately.
    pub fn fast_forward(ui: &egui::Ui, g: &mut Graph<N, E, Ty, Ix, Dn, De>, steps: u32)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
        S: LayoutState,
        L: Layout<S>,
    {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            g,
            steps,
            None,
            |_s| None,
            |_s, _tok| {},
        );
    }

    /// Advance the active layout by up to `target_steps`, but stop early if `max_millis` has elapsed.
    /// Returns the number of steps actually performed.
    pub fn fast_forward_budgeted(
        ui: &egui::Ui,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        target_steps: u32,
        max_millis: u64,
    ) -> u32
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
        S: LayoutState,
        L: Layout<S>,
    {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            g,
            target_steps,
            Some(max_millis),
            |_s| None,
            |_s, _tok| {},
        )
    }

    /// Run simulation steps until the average node displacement drops below `epsilon`
    /// or `max_steps` is reached. Returns (`steps_done`, `last_avg_disp`).
    pub fn fast_forward_until_stable(
        ui: &egui::Ui,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
    ) -> (u32, f32)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
        S: LayoutState,
        L: Layout<S>,
    {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            g,
            epsilon,
            max_steps,
            None,
            |_s| None, // no internal metric available in general case
            |_s| None,
            |_s, _tok| {},
        )
    }

    /// Budgeted variant of `fast_forward_until_stable`.
    pub fn fast_forward_until_stable_budgeted(
        ui: &egui::Ui,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
        max_millis: u64,
    ) -> (u32, f32)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
        S: LayoutState,
        L: Layout<S>,
    {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            g,
            epsilon,
            max_steps,
            Some(max_millis),
            |_s| None,
            |_s| None,
            |_s, _tok| {},
        )
    }
    // See a separate impl with an explicit lifetime for the `with_events` method.

    fn sync_layout(&mut self, ui: &mut Ui) {
        let state = ui.data_mut(|data| {
            data.get_persisted::<S>(Id::new(KEY_LAYOUT))
                .unwrap_or_default()
        });

        let mut layout = L::from_state(state);
        layout.next(self.g, ui);
        let new_state = layout.state();

        ui.data_mut(|data| {
            data.insert_persisted(Id::new(KEY_LAYOUT), new_state);
        });
    }

    fn sync_state(&mut self, meta: &mut Metadata) {
        let mut selected_nodes = Vec::new();
        let mut selected_edges = Vec::new();
        let mut dragged = None;

        meta.reset_bounds();
        self.g.nodes_iter().for_each(|(idx, n)| {
            if n.dragged() {
                dragged = Some(idx);
            }
            if n.selected() {
                selected_nodes.push(idx);
            }

            meta.process_bounds(n);
        });

        self.g.edges_iter().for_each(|(idx, e)| {
            if e.selected() {
                selected_edges.push(idx);
            }
            if let Some((start_idx, end_idx)) = self.g.edge_endpoints(e.id()) {
                if let (Some(start), Some(end)) = (self.g.node(start_idx), self.g.node(end_idx)) {
                    if let Some((min, max)) = e.display().extra_bounds(start, end) {
                        meta.expand_bounds(min, max);
                    }
                }
            }
        });

        self.g.set_selected_nodes(selected_nodes);
        self.g.set_selected_edges(selected_edges);
        self.g.set_dragged_node(dragged);
        self.g.set_bounds(meta.graph_bounds());
    }

    /// Fits the graph to the screen if it is the first frame or
    /// fit to screen setting is enabled;
    fn handle_fit_to_screen(&self, r: &Response, meta: &mut Metadata) {
        if !meta.first_frame && !self.settings_navigation.fit_to_screen_enabled {
            return;
        }

        self.fit_to_screen(&r.rect, meta);
    }

    fn handle_click(&mut self, resp: &Response, meta: &mut Metadata, eff: EffectiveInteraction) {
        if !resp.clicked() && !resp.double_clicked() {
            return;
        }

        let clickable = eff.node_clicking
            || eff.node_selection
            || eff.node_selection_multi
            || eff.edge_clicking
            || eff.edge_selection
            || eff.edge_selection_multi;

        if !(clickable) {
            return;
        }

        let Some(cursor_pos) = resp.hover_pos() else {
            return;
        };
        let found_edge = self.g.edge_by_screen_pos(meta, cursor_pos);
        let found_node = self.g.node_by_screen_pos(meta, cursor_pos);
        if found_node.is_none() && found_edge.is_none() {
            // click on empty space
            let nodes_selectable = eff.node_selection || eff.node_selection_multi;
            if nodes_selectable {
                self.deselect_all_nodes();
            }

            let edges_selectable = eff.edge_selection || eff.edge_selection_multi;
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
                self.handle_node_double_click(idx, eff);
                return;
            }
            self.handle_node_click(idx, eff);
            return;
        }

        if let Some(edge_idx) = found_edge {
            self.handle_edge_click(edge_idx, eff);
        }
    }

    fn handle_node_double_click(&mut self, idx: NodeIndex<Ix>, eff: EffectiveInteraction) {
        if !eff.node_clicking {
            return;
        }

        if eff.node_clicking {
            self.set_node_double_clicked(idx);
        }
    }

    fn handle_node_click(&mut self, idx: NodeIndex<Ix>, eff: EffectiveInteraction) {
        if !eff.node_clicking && !eff.node_selection {
            return;
        }

        if eff.node_clicking {
            self.set_node_clicked(idx);
        }

        if !eff.node_selection {
            return;
        }

        let n = self.g.node(idx).unwrap();
        if n.selected() {
            self.deselect_node(idx);
            return;
        }

        if !eff.node_selection_multi {
            self.deselect_all();
        }

        self.select_node(idx);
    }

    fn handle_edge_click(&mut self, idx: EdgeIndex<Ix>, eff: EffectiveInteraction) {
        if !eff.edge_clicking && !eff.edge_selection {
            return;
        }

        if eff.edge_clicking {
            self.set_edge_clicked(idx);
        }

        if !eff.edge_selection {
            return;
        }

        let e = self.g.edge(idx).unwrap();
        if e.selected() {
            self.deselect_edge(idx);
            return;
        }

        if !eff.edge_selection_multi {
            self.deselect_all();
        }

        self.select_edge(idx);
    }

    fn handle_node_drag(
        &mut self,
        resp: &Response,
        meta: &mut Metadata,
        eff: EffectiveInteraction,
    ) {
        if !eff.dragging {
            return;
        }

        // Immediately mark a node as dragged on pointer-down over it, and end on release.
        let node_hover_index = match resp.hover_pos() {
            Some(hover_pos) => self.g.node_by_screen_pos(meta, hover_pos),
            None => None,
        };
        if resp.is_pointer_button_down_on() {
            if self.g.dragged_node().is_none() {
                if let Some(idx) = node_hover_index {
                    self.set_drag_start(idx);
                    self.g.set_dragged_node(Some(idx));
                }
            }
        } else if !resp.is_pointer_button_down_on() {
            if let Some(dragged_idx) = self.g.dragged_node() {
                self.set_drag_end(dragged_idx);
                self.g.set_dragged_node(None);
            }
        }

        if !resp.dragged_by(PointerButton::Primary)
            && !resp.drag_started_by(PointerButton::Primary)
            && !resp.drag_stopped_by(PointerButton::Primary)
        {
            return;
        }

        // If a drag started and no node is currently marked as dragged (e.g., started outside a node),
        // try to start dragging the node under the cursor once. Otherwise skip to avoid double-starting.
        if resp.drag_started() && self.g.dragged_node().is_none() {
            if let Some(pos) = resp.hover_pos() {
                if let Some(idx) = self.g.node_by_screen_pos(meta, pos) {
                    self.set_drag_start(idx);
                    self.g.set_dragged_node(Some(idx));
                }
            }
        }

        // handle mouse drag
        if resp.dragged()
            && self.g.dragged_node().is_some()
            && (resp.drag_delta().x.abs() > 0. || resp.drag_delta().y.abs() > 0.)
        {
            let n_idx_dragged = self.g.dragged_node().unwrap();
            let delta_in_graph_coords = resp.drag_delta() / meta.zoom;
            self.move_node(n_idx_dragged, delta_in_graph_coords);
        }

        // compensate movement of the node which is not caused by dragging
        if let Some(n_idx_dragged) = self.g.dragged_node() {
            if let Some(mouse_pos) = resp.hover_pos() {
                if let Some(node) = self.g.node(n_idx_dragged) {
                    let node_pos = node.location() * meta.zoom + meta.pan;
                    let delta = mouse_pos - node_pos;

                    self.move_node(n_idx_dragged, delta / meta.zoom);
                }
            }
        }

        if resp.drag_stopped() && self.g.dragged_node().is_some() {
            let n_idx = self.g.dragged_node().unwrap();
            self.set_drag_end(n_idx);
        }
    }

    fn fit_to_screen(&self, rect: &Rect, meta: &mut Metadata) {
        let raw_bounds = meta.graph_bounds();
        let (mut min, mut max) = (raw_bounds.min, raw_bounds.max);
        let invalid_bounds = !min.x.is_finite()
            || !min.y.is_finite()
            || !max.x.is_finite()
            || !max.y.is_finite()
            || min.x > max.x
            || min.y > max.y;
        if invalid_bounds {
            min = Pos2::new(-0.5, -0.5);
            max = Pos2::new(0.5, 0.5);
        }
        let mut diag: Vec2 = max - min;
        if !diag.x.is_finite() || !diag.y.is_finite() || diag.x <= 0.0 || diag.y <= 0.0 {
            diag = Vec2::new(1., 1.);
        }
        let graph_size = diag * (1. + self.settings_navigation.fit_to_screen_padding);
        let (width, height) = (graph_size.x.max(1e-3), graph_size.y.max(1e-3));
        let canvas_size = rect.size();
        let (canvas_width, canvas_height) = (canvas_size.x, canvas_size.y);
        let zoom_x = (canvas_width / width).abs();
        let zoom_y = (canvas_height / height).abs();
        let mut new_zoom = zoom_x.min(zoom_y);
        if !new_zoom.is_finite() || new_zoom <= 0.0 {
            new_zoom = 1.0;
        }
        let zoom_delta = new_zoom / meta.zoom - 1.0;
        self.zoom(rect, zoom_delta, None, meta);
        let graph_center = (min.to_vec2() + max.to_vec2()) / 2.0;
        let new_pan = rect.center().to_vec2() - graph_center * new_zoom;
        self.set_pan(new_pan, meta);
    }

    fn handle_navigation(
        &self,
        ui: &Ui,
        resp: &Response,
        meta: &mut Metadata,
        eff: EffectiveInteraction,
    ) {
        if !meta.first_frame {
            meta.pan += resp.rect.left_top() - meta.top_left;
        }
        meta.top_left = resp.rect.left_top();

        self.handle_zoom(ui, resp, meta, eff);
        self.handle_pan(resp, meta, eff);
    }

    fn handle_zoom(
        &self,
        ui: &Ui,
        resp: &Response,
        meta: &mut Metadata,
        _eff: EffectiveInteraction,
    ) {
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

    fn handle_pan(&self, resp: &Response, meta: &mut Metadata, _eff: EffectiveInteraction) {
        if !self.settings_navigation.zoom_and_pan_enabled {
            return;
        }

        if (resp.dragged_by(PointerButton::Middle) || resp.dragged_by(PointerButton::Primary))
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

    #[allow(unused_variables, clippy::unused_self)]
    fn set_node_clicked(&self, idx: NodeIndex<Ix>) {
        #[cfg(feature = "events")]
        self.publish_event(Event::NodeClick(PayloadNodeClick { id: idx.index() }));
    }

    #[allow(unused_variables, clippy::unused_self)]
    fn set_node_double_clicked(&self, idx: NodeIndex<Ix>) {
        #[cfg(feature = "events")]
        self.publish_event(Event::NodeDoubleClick(PayloadNodeDoubleClick {
            id: idx.index(),
        }));
    }

    #[allow(unused_variables, clippy::unused_self)]
    fn set_edge_clicked(&self, idx: EdgeIndex<Ix>) {
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
        for idx in selected_nodes {
            self.deselect_node(idx);
        }
    }

    fn deselect_all_edges(&mut self) {
        let selected_edges = self.g.selected_edges().to_vec();
        for idx in selected_edges {
            self.deselect_edge(idx);
        }
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

    #[allow(unused_variables, clippy::unused_self)]
    fn set_pan(&self, new_pan: Vec2, meta: &mut Metadata) {
        let diff = new_pan - meta.pan;
        meta.pan = new_pan;

        #[cfg(feature = "events")]
        self.publish_event(Event::Pan(PayloadPan {
            diff: diff.into(),
            new_pan: new_pan.into(),
        }));
    }

    #[allow(unused_variables, clippy::unused_self)]
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

// Force-run variants available when the layout state supports animation toggling.
impl<N, E, Ty, Ix, Dn, De, S, L> GraphView<'_, N, E, Ty, Ix, Dn, De, S, L>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
    S: layouts::AnimatedState + LayoutState,
    L: Layout<S>,
{
    /// Advance simulation even if paused by temporarily forcing `running = true`.
    pub fn fast_forward_force_run(ui: &egui::Ui, g: &mut Graph<N, E, Ty, Ix, Dn, De>, steps: u32) {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            g,
            steps,
            None,
            |s| {
                let prev = Some(s.is_running());
                s.set_running(true);
                prev
            },
            |s, prev| {
                if let Some(p) = prev {
                    s.set_running(p);
                }
            },
        );
    }

    /// Budgeted variant of `fast_forward_force_run`.
    pub fn fast_forward_budgeted_force_run(
        ui: &egui::Ui,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        target_steps: u32,
        max_millis: u64,
    ) -> u32 {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            g,
            target_steps,
            Some(max_millis),
            |s| {
                let prev = Some(s.is_running());
                s.set_running(true);
                prev
            },
            |s, prev| {
                if let Some(p) = prev {
                    s.set_running(p);
                }
            },
        )
    }

    /// Until-stable variant that forces running during the operation.
    pub fn fast_forward_until_stable_force_run(
        ui: &egui::Ui,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
    ) -> (u32, f32) {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            g,
            epsilon,
            max_steps,
            None,
            super::layouts::AnimatedState::last_avg_displacement,
            |s| {
                let prev = Some(s.is_running());
                s.set_running(true);
                prev
            },
            |s, prev| {
                if let Some(p) = prev {
                    s.set_running(p);
                }
            },
        )
    }

    /// Budgeted until-stable variant with forced running.
    pub fn fast_forward_until_stable_budgeted_force_run(
        ui: &egui::Ui,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
        max_millis: u64,
    ) -> (u32, f32) {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            g,
            epsilon,
            max_steps,
            Some(max_millis),
            super::layouts::AnimatedState::last_avg_displacement,
            |s| {
                let prev = Some(s.is_running());
                s.set_running(true);
                prev
            },
            |s, prev| {
                if let Some(p) = prev {
                    s.set_running(p);
                }
            },
        )
    }
}
