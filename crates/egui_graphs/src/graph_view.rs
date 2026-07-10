use std::marker::PhantomData;

use crate::{
    draw::{drawer::Drawer, DrawContext},
    graph_view_response::{GraphChange, GraphViewResponse},
    layouts::{self, Layout, LayoutState},
    metadata::{reset_metadata, MetadataFrame, MetadataInstance},
    settings::{SettingsInteraction, SettingsNavigation, SettingsStyle},
    DisplayEdge, DisplayNode, Graph,
};

use egui::{Id, PointerButton, Pos2, Rect, Response, Sense, Ui, Vec2};
use web_time::Instant;

use petgraph::graph::EdgeIndex;
use petgraph::graph::IndexType;
use petgraph::{stable_graph::NodeIndex, EdgeType};

// Shared cores to avoid duplication across general and force-run variants.
fn ff_steps_core<N, E, Ty, Ix, Dn, De, S, L, Pre, Post>(
    ui: &mut egui::Ui,
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    target_steps: u32,
    budget_millis: Option<u64>,
    pre_toggle: Pre,
    post_toggle: Post,
    id: Option<String>,
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
    let mut state = get_layout_state::<S>(ui, id.clone());
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
    set_layout_state::<S>(ui, new_state, id);
    done
}

#[allow(clippy::too_many_arguments)]
fn ff_until_stable_core<N, E, Ty, Ix, Dn, De, S, L, Metric, Pre, Post>(
    ui: &mut egui::Ui,
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    epsilon: f32,
    max_steps: u32,
    budget_millis: Option<u64>,
    metric: Metric,
    pre_toggle: Pre,
    post_toggle: Post,
    id: Option<String>,
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

    let mut state = get_layout_state::<S>(ui, id.clone());
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
    set_layout_state::<S>(ui, new_state, id);
    (
        steps_done,
        if last_avg.is_finite() { last_avg } else { 0.0 },
    )
}

pub type DefaultGraphView = GraphView<layouts::random::State, layouts::random::Random>;

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

/// Configures and renders an interactive graph.
///
/// Pass the graph to [`GraphView::show`]. The returned [`GraphViewResponse`] contains both
/// the regular egui response and graph-specific changes produced during that frame.
pub struct GraphView<S = layouts::random::State, L = layouts::random::Random>
where
    S: LayoutState,
    L: Layout<S>,
{
    settings_interaction: SettingsInteraction,
    settings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,

    custom_id: Option<String>,

    _marker: PhantomData<(L, S)>,
}

impl<S, L> Default for GraphView<S, L>
where
    S: LayoutState,
    L: Layout<S>,
{
    fn default() -> Self {
        Self {
            settings_style: SettingsStyle::default(),
            settings_interaction: SettingsInteraction::default(),
            settings_navigation: SettingsNavigation::default(),
            custom_id: None,
            _marker: PhantomData,
        }
    }
}

struct GraphViewFrame<'g, N, E, Ty, Ix, Nd, Ed, S, L>
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
    g: &'g mut Graph<N, E, Ty, Ix, Nd, Ed>,
    settings_interaction: SettingsInteraction,
    settings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,
    custom_id: Option<String>,
    changes: Vec<GraphChange<Ix>>,
    _marker: PhantomData<(L, S)>,
}

struct ViewState {
    pub frame: MetadataFrame,
    pub instance: MetadataInstance,
    pub sync: crate::metadata::MetadataSync,
    pub instance_id: String,
}

impl ViewState {
    fn load(
        ui: &mut Ui,
        widget_id: Id,
        custom_id: &Option<String>,
        fallback_top_left: Pos2,
    ) -> Self {
        let frame = MetadataFrame::new(custom_id.clone()).load(ui);
        let instance = MetadataInstance::load(ui, widget_id, custom_id, fallback_top_left);
        let sync = crate::metadata::MetadataSync::load(ui, custom_id);
        let instance_id =
            crate::metadata::instance_key_string(widget_id, custom_id.clone(), "instance");
        Self {
            frame,
            instance,
            sync,
            instance_id,
        }
    }

    fn save(&self, ui: &mut Ui, widget_id: Id, custom_id: &Option<String>) {
        self.frame.clone().save(ui);
        self.instance.save(ui, widget_id, custom_id);
        self.sync.save(ui, custom_id);
    }
}

impl<N, E, Ty, Ix, Nd, Ed, S, L> GraphViewFrame<'_, N, E, Ty, Ix, Nd, Ed, S, L>
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
    fn show(mut self, ui: &mut Ui) -> GraphViewResponse<Ix> {
        // Measure layout step time
        let t0 = Instant::now();
        self.sync_layout(ui);
        let step_ms = t0.elapsed().as_secs_f32() * 1000.0;

        // Compute effective interactions once per frame
        let eff = self.effective();

        let (resp, p) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // Load both shared (per custom_id) and local (per widget instance) state once
        let mut view = ViewState::load(ui, resp.id, &self.custom_id, resp.rect.left_top());
        self.sync_state(&mut view.frame);
        // Apply per-instance pan compensation before hover so hit-testing uses the correct transform.
        if view.instance.last_top_left != resp.rect.left_top() && !view.instance.first_frame_pending
        {
            view.frame.pan += resp.rect.left_top() - view.instance.last_top_left;
        }
        view.instance.last_top_left = resp.rect.left_top();

        // Hover detection and cursor update happens as early as possible using current input state
        self.handle_hover(ui, &resp, &mut view, eff);
        self.handle_fit_to_screen(&resp, &mut view.frame, &mut view.instance);

        // Handle node drag before navigation so pan doesn't kick in on the first frame
        // when starting a node drag.
        self.handle_node_drag(&resp, &mut view, eff);

        self.handle_navigation(ui, &resp, &mut view.frame, eff);
        self.handle_click(&resp, &mut view.frame, eff);

        // Measure draw time (exclude layout step): start after layout, stop after draw
        let t_draw0 = Instant::now();
        // Use a draw-time metadata adjusted to screen coordinates by adding the widget's top-left offset.
        let mut meta_draw = view.frame.clone();
        meta_draw.pan += resp.rect.left_top().to_vec2();

        Drawer::<N, E, Ty, Ix, Nd, Ed, S, L>::new(
            self.g,
            &DrawContext {
                ctx: ui.ctx(),
                painter: &p,
                meta: &meta_draw,
                is_directed: self.g.is_directed(),
                style: &self.settings_style,
            },
        )
        .draw();
        let draw_ms = t_draw0.elapsed().as_secs_f32() * 1000.0;

        view.frame.last_step_time_ms = step_ms;
        view.frame.last_draw_time_ms = draw_ms;

        // Mark end of first frame for this instance
        view.instance.first_frame_pending = false;

        // Consolidated writes at the end of the frame
        view.save(ui, resp.id, &self.custom_id);

        ui.ctx().request_repaint();

        GraphViewResponse {
            response: resp,
            changes: self.changes,
        }
    }
}

// Configuration and rendering entry point.
impl<S, L> GraphView<S, L>
where
    S: LayoutState,
    L: Layout<S>,
{
    /// Creates a graph view with default navigation, interaction, and style settings.
    /// To customize navigation and interactions use `with_interactions` and `with_navigations` methods.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders `graph` and returns both the egui response and graph-specific changes
    /// produced during this frame.
    pub fn show<N, E, Ty, Ix, Dn, De>(
        self,
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
    ) -> GraphViewResponse<Ix>
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        GraphViewFrame::<N, E, Ty, Ix, Dn, De, S, L> {
            g: graph,
            settings_interaction: self.settings_interaction,
            settings_navigation: self.settings_navigation,
            settings_style: self.settings_style,
            custom_id: self.custom_id,
            changes: Vec::new(),
            _marker: PhantomData,
        }
        .show(ui)
    }

    /// Makes the view interactive according to the provided settings.
    pub fn with_interactions(mut self, settings_interaction: &SettingsInteraction) -> Self {
        self.settings_interaction = settings_interaction.clone();
        self
    }

    /// Modifies default navigation settings.
    pub fn with_navigations(mut self, settings_navigation: &SettingsNavigation) -> Self {
        self.settings_navigation = settings_navigation.clone();
        self
    }

    /// Modifies default style settings.
    pub fn with_styles(mut self, settings_style: &SettingsStyle) -> Self {
        self.settings_style = settings_style.clone();
        self
    }

    /// Sets a custom ID used for layout and view metadata persistence.
    pub fn with_id(mut self, custom_id: Option<String>) -> Self {
        self.custom_id = custom_id;
        self
    }

    /// Advance the active layout simulation by a fixed number of steps immediately.
    pub fn fast_forward<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        steps: u32,
        id: Option<String>,
    ) where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            graph,
            steps,
            None,
            |_state| None,
            |_state, _token| {},
            id,
        );
    }

    /// Advance the active layout by up to `target_steps`, stopping at `max_millis`.
    pub fn fast_forward_budgeted<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        target_steps: u32,
        max_millis: u64,
        id: Option<String>,
    ) -> u32
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            graph,
            target_steps,
            Some(max_millis),
            |_state| None,
            |_state, _token| {},
            id,
        )
    }

    /// Run simulation steps until average node displacement is below `epsilon`
    /// or `max_steps` is reached.
    pub fn fast_forward_until_stable<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
        id: Option<String>,
    ) -> (u32, f32)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            graph,
            epsilon,
            max_steps,
            None,
            |_state| None,
            |_state| None,
            |_state, _token| {},
            id,
        )
    }

    /// Budgeted variant of [`Self::fast_forward_until_stable`].
    pub fn fast_forward_until_stable_budgeted<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
        max_millis: u64,
        id: Option<String>,
    ) -> (u32, f32)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            graph,
            epsilon,
            max_steps,
            Some(max_millis),
            |_state| None,
            |_state| None,
            |_state, _token| {},
            id,
        )
    }
}

impl<N, E, Ty, Ix, Dn, De, S, L> GraphViewFrame<'_, N, E, Ty, Ix, Dn, De, S, L>
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
        view: &mut ViewState,
        eff: EffectiveInteraction,
    ) {
        let meta = &mut view.frame;

        if self.g.dragged_node().is_some() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }

        if !eff.hover {
            return;
        }

        // Synchronized hover: claim on hover, only owner can clear.
        let is_owner =
            matches!(view.sync.hover_owner.as_deref(), Some(owner) if owner == view.instance_id);

        // Convert to widget-local coordinates for hit-testing.
        let hovered_now = if let Some(pos) = resp.hover_pos() {
            self.g.node_by_screen_pos(meta, self.local_pos(resp, pos))
        } else {
            None
        };

        if hovered_now.is_some() {
            // Claim ownership when actually hovering in this instance.
            view.sync.hover_owner = Some(view.instance_id.clone());
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        } else if !is_owner {
            // Do not clear hover if we are not the owner.
            return;
        } else {
            // We are the owner but no longer hovering: release ownership.
            view.sync.hover_owner = None;
        }

        let prev = self.g.hovered_node();
        if hovered_now != prev {
            if let Some(prev_idx) = prev {
                self.record_change(GraphChange::NodeHoverExited { node: prev_idx });
                if let Some(n) = self.g.node_mut(prev_idx) {
                    n.set_hovered(false);
                }
            }
            if let Some(cur_idx) = hovered_now {
                self.record_change(GraphChange::NodeHoverEntered { node: cur_idx });
                if let Some(n) = self.g.node_mut(cur_idx) {
                    n.set_hovered(true);
                }
            }
            self.g.set_hovered_node(hovered_now);
        }
    }

    fn sync_layout(&mut self, ui: &mut Ui) {
        let id = self.custom_id.clone();

        let state = S::load(ui, id.clone());

        let mut layout = L::from_state(state);
        layout.next(self.g, ui);
        let new_state = layout.state();

        new_state.save(ui, id);
    }

    fn sync_state(&mut self, meta: &mut MetadataFrame) {
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
    fn handle_fit_to_screen(
        &mut self,
        r: &Response,
        meta: &mut MetadataFrame,
        instance: &mut MetadataInstance,
    ) {
        // Fit if this instance is on its first frame, or if the global setting is enabled.
        if !(instance.first_frame_pending || self.settings_navigation.fit_to_screen_enabled) {
            return;
        }

        // Use a local rect (origin at 0,0) for fit-to-screen calculations.
        let local_rect = Rect::from_min_size(Pos2::ZERO, r.rect.size());
        self.fit_to_screen(&local_rect, meta);

        // Mark this instance as having completed its first-frame fit.
        instance.first_frame_pending = false;
    }

    fn handle_click(
        &mut self,
        resp: &Response,
        meta: &mut MetadataFrame,
        eff: EffectiveInteraction,
    ) {
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
        // Convert to widget-local coordinates.
        let local_pos = self.local_pos(resp, cursor_pos);
        let found_edge = self.g.edge_by_screen_pos(meta, local_pos);
        let found_node = self.g.node_by_screen_pos(meta, local_pos);
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
        view: &mut ViewState,
        eff: EffectiveInteraction,
    ) {
        let meta = &mut view.frame;

        if !eff.dragging {
            return;
        }

        // Determine ownership of the drag for shared-id scenarios.
        let is_owner =
            matches!(view.sync.drag_owner.as_deref(), Some(owner) if owner == view.instance_id);

        // If another instance owns the drag, ignore all drag handling in this instance.
        if view.sync.drag_owner.is_some() && !is_owner {
            return;
        }

        // Immediately mark a node as dragged on pointer-down over it, and end on release.
        let node_hover_index = match resp.hover_pos() {
            Some(hover_pos) => self
                .g
                .node_by_screen_pos(meta, self.local_pos(resp, hover_pos)),
            None => None,
        };

        if resp.is_pointer_button_down_on() {
            if self.g.dragged_node().is_none() {
                if let Some(idx) = node_hover_index {
                    self.set_drag_start(idx);
                    self.g.set_dragged_node(Some(idx));
                    // Acquire ownership for this instance
                    view.sync.drag_owner = Some(view.instance_id.clone());
                }
            }
        } else if !resp.is_pointer_button_down_on() && self.g.dragged_node().is_some() && is_owner {
            let dragged_idx = self.g.dragged_node().unwrap();
            self.set_drag_end(dragged_idx);
            self.g.set_dragged_node(None);
            // Release ownership
            view.sync.drag_owner = None;
        }

        // From here, only the owner continues to process drag deltas and compensation.
        if !matches!(view.sync.drag_owner.as_deref(), Some(owner) if owner == view.instance_id) {
            return;
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
                let mouse_pos_local = self.local_pos(resp, mouse_pos);
                if let Some(node) = self.g.node(n_idx_dragged) {
                    let node_pos = node.location() * meta.zoom + meta.pan;
                    let delta = mouse_pos_local - node_pos;

                    self.move_node(n_idx_dragged, delta / meta.zoom);
                }
            }
        }

        if resp.drag_stopped() && self.g.dragged_node().is_some() {
            let n_idx = self.g.dragged_node().unwrap();
            self.set_drag_end(n_idx);
            self.g.set_dragged_node(None);
            // Release ownership on drag stop
            view.sync.drag_owner = None;
        }
    }

    fn fit_to_screen(&mut self, rect: &Rect, meta: &mut MetadataFrame) {
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
        &mut self,
        ui: &Ui,
        resp: &Response,
        meta: &mut MetadataFrame,
        eff: EffectiveInteraction,
    ) {
        self.handle_zoom(ui, resp, meta, eff);
        self.handle_pan(resp, meta, eff);
    }

    fn handle_zoom(
        &mut self,
        ui: &Ui,
        resp: &Response,
        meta: &mut MetadataFrame,
        _eff: EffectiveInteraction,
    ) {
        if !self.settings_navigation.zoom_and_pan_enabled {
            return;
        }

        let Some(pointer_pos) = resp.hover_pos() else {
            return;
        };

        ui.input(|i| {
            let delta = i.zoom_delta();
            if delta == 1. {
                return;
            }

            let step = self.settings_navigation.zoom_speed * (delta - 1.).signum();
            let local_center = Some(self.local_pos(resp, pointer_pos));
            // Use a local rect (origin at 0,0) for zoom center math.
            let local_rect = Rect::from_min_size(Pos2::ZERO, resp.rect.size());
            self.zoom(&local_rect, step, local_center, meta);
        });
    }

    fn handle_pan(
        &mut self,
        resp: &Response,
        meta: &mut MetadataFrame,
        _eff: EffectiveInteraction,
    ) {
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

    /// Convert a screen-space position to widget-local position
    fn local_pos(&self, resp: &Response, p: Pos2) -> Pos2 {
        (p - resp.rect.left_top()).to_pos2()
    }

    /// Zooms the graph by the given delta. It also compensates with pan to keep the zoom center in the same place.
    fn zoom(
        &mut self,
        rect: &Rect,
        delta: f32,
        zoom_center: Option<Pos2>,
        meta: &mut MetadataFrame,
    ) {
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
        self.record_change(GraphChange::NodeSelected { node: idx });
    }

    fn deselect_node(&mut self, idx: NodeIndex<Ix>) {
        let n = self.g.node_mut(idx).unwrap();
        n.set_selected(false);
        self.record_change(GraphChange::NodeDeselected { node: idx });
    }

    fn set_node_clicked(&mut self, idx: NodeIndex<Ix>) {
        self.record_change(GraphChange::NodeClicked { node: idx });
    }

    fn set_node_double_clicked(&mut self, idx: NodeIndex<Ix>) {
        self.record_change(GraphChange::NodeDoubleClicked { node: idx });
    }

    fn set_edge_clicked(&mut self, idx: EdgeIndex<Ix>) {
        self.record_change(GraphChange::EdgeClicked { edge: idx });
    }

    fn select_edge(&mut self, idx: EdgeIndex<Ix>) {
        let e = self.g.edge_mut(idx).unwrap();
        e.set_selected(true);
        self.record_change(GraphChange::EdgeSelected { edge: idx });
    }

    fn deselect_edge(&mut self, idx: EdgeIndex<Ix>) {
        let e = self.g.edge_mut(idx).unwrap();
        e.set_selected(false);
        self.record_change(GraphChange::EdgeDeselected { edge: idx });
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
        self.record_change(GraphChange::NodeMoved {
            node: idx,
            delta,
            new_position: new_loc,
        });
    }

    fn set_drag_start(&mut self, idx: NodeIndex<Ix>) {
        let n = self.g.node_mut(idx).unwrap();
        n.set_dragged(true);
        self.record_change(GraphChange::NodeDragStarted { node: idx });
    }

    fn set_drag_end(&mut self, idx: NodeIndex<Ix>) {
        let n = self.g.node_mut(idx).unwrap();
        n.set_dragged(false);
        self.record_change(GraphChange::NodeDragEnded { node: idx });
    }

    fn set_pan(&mut self, new_pan: Vec2, meta: &mut MetadataFrame) {
        let delta = new_pan - meta.pan;
        if delta == Vec2::ZERO {
            return;
        }

        meta.pan = new_pan;
        self.record_change(GraphChange::Panned { delta, new_pan });
    }

    fn set_zoom(&mut self, new_zoom: f32, meta: &mut MetadataFrame) {
        let delta = new_zoom - meta.zoom;
        if delta == 0. {
            return;
        }

        meta.zoom = new_zoom;
        self.record_change(GraphChange::Zoomed { delta, new_zoom });
    }

    fn record_change(&mut self, change: GraphChange<Ix>) {
        self.changes.push(change);
    }
}

// Force-run variants available when the layout state supports animation toggling.
impl<S, L> GraphView<S, L>
where
    S: layouts::AnimatedState + LayoutState,
    L: Layout<S>,
{
    /// Advance simulation even if paused by temporarily forcing `running = true`.
    pub fn fast_forward_force_run<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        steps: u32,
        id: Option<String>,
    ) where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            graph,
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
            id,
        );
    }

    /// Budgeted variant of `fast_forward_force_run`.
    pub fn fast_forward_budgeted_force_run<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        target_steps: u32,
        max_millis: u64,
        id: Option<String>,
    ) -> u32
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_steps_core::<N, E, Ty, Ix, Dn, De, S, L, _, _>(
            ui,
            graph,
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
            id,
        )
    }

    /// Until-stable variant that forces running during the operation.
    pub fn fast_forward_until_stable_force_run<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
        id: Option<String>,
    ) -> (u32, f32)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            graph,
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
            id,
        )
    }

    /// Budgeted until-stable variant with forced running.
    pub fn fast_forward_until_stable_budgeted_force_run<N, E, Ty, Ix, Dn, De>(
        ui: &mut Ui,
        graph: &mut Graph<N, E, Ty, Ix, Dn, De>,
        epsilon: f32,
        max_steps: u32,
        max_millis: u64,
        id: Option<String>,
    ) -> (u32, f32)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        ff_until_stable_core::<N, E, Ty, Ix, Dn, De, S, L, _, _, _>(
            ui,
            graph,
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
            id,
        )
    }
}

/// Helper to reset both [`MetadataFrame`] and [`Layout`] cache. Can be useful when you want to change layout in runtime
pub fn reset<S: LayoutState>(ui: &mut Ui, id: Option<String>) {
    reset_metadata(ui, id.clone());
    reset_layout::<S>(ui, id.clone());
}

/// Returns the latest per-frame performance metrics stored in metadata.
pub fn get_metrics(ui: &egui::Ui, id: Option<String>) -> (f32, f32) {
    let m = MetadataFrame::new(id).load(ui);
    (m.last_step_time_ms, m.last_draw_time_ms)
}

/// Resets [`Layout`] state
pub fn reset_layout<S: LayoutState>(ui: &mut Ui, id: Option<String>) {
    S::default().save(ui, id);
}

/// Loads current persisted layout state (or default if none). Useful for external UI panels.
pub fn get_layout_state<S: LayoutState>(ui: &egui::Ui, id: Option<String>) -> S {
    S::load(ui, id)
}

/// Persists a new layout state so that on the next frame it will be applied.
pub fn set_layout_state<S: LayoutState>(ui: &mut egui::Ui, state: S, id: Option<String>) {
    state.save(ui, id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DefaultEdgeShape, DefaultNodeShape};
    use petgraph::{
        stable_graph::{DefaultIx, StableGraph},
        Directed,
    };

    type TestFrame<'g> = GraphViewFrame<
        'g,
        (),
        (),
        Directed,
        DefaultIx,
        DefaultNodeShape,
        DefaultEdgeShape,
        layouts::random::State,
        layouts::random::Random,
    >;

    fn test_frame(graph: &mut Graph) -> TestFrame<'_> {
        GraphViewFrame {
            g: graph,
            settings_interaction: SettingsInteraction::default(),
            settings_navigation: SettingsNavigation::default(),
            settings_style: SettingsStyle::default(),
            custom_id: None,
            changes: Vec::new(),
            _marker: PhantomData,
        }
    }

    #[test]
    fn records_typed_changes_in_occurrence_order_without_coalescing() {
        let stable = StableGraph::<(), ()>::default();
        let mut graph: Graph = Graph::from(&stable);
        let node = graph.add_node(());
        let mut frame = test_frame(&mut graph);

        frame.set_node_clicked(node);
        frame.select_node(node);
        frame.move_node(node, Vec2::new(1.0, 2.0));
        frame.move_node(node, Vec2::new(3.0, 4.0));

        assert_eq!(
            frame.changes,
            vec![
                GraphChange::NodeClicked { node },
                GraphChange::NodeSelected { node },
                GraphChange::NodeMoved {
                    node,
                    delta: Vec2::new(1.0, 2.0),
                    new_position: Pos2::new(1.0, 2.0),
                },
                GraphChange::NodeMoved {
                    node,
                    delta: Vec2::new(3.0, 4.0),
                    new_position: Pos2::new(4.0, 6.0),
                },
            ]
        );
    }

    #[test]
    fn show_returns_only_changes_from_the_current_frame() {
        let ctx = egui::Context::default();
        let stable = StableGraph::<(), ()>::default();
        let mut graph: Graph = Graph::from(&stable);
        graph.add_node(());
        let navigation = SettingsNavigation::default().with_fit_to_screen_enabled(false);

        let mut first_changes = Vec::new();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            first_changes = DefaultGraphView::new()
                .with_navigations(&navigation)
                .show(ui, &mut graph)
                .changes;
        });

        assert!(first_changes
            .iter()
            .any(|change| matches!(change, GraphChange::Panned { .. })));
        assert!(first_changes
            .iter()
            .any(|change| matches!(change, GraphChange::Zoomed { .. })));

        let mut second_changes = Vec::new();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            second_changes = DefaultGraphView::new()
                .with_navigations(&navigation)
                .show(ui, &mut graph)
                .changes;
        });

        assert!(second_changes.is_empty());
    }

    #[test]
    fn show_infers_a_non_default_graph_index_type() {
        let ctx = egui::Context::default();
        let stable = StableGraph::<(), (), Directed, u16>::default();
        let mut graph: Graph<(), (), Directed, u16> = Graph::from(&stable);
        graph.add_node(());

        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let response: GraphViewResponse<u16> = DefaultGraphView::new().show(ui, &mut graph);
            assert!(!response.changes.is_empty());
        });
    }
}
