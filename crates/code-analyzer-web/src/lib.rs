use eframe::wasm_bindgen::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no global window exists")
            .document()
            .expect("should have a document on window");
        
        let canvas = document
            .get_element_by_id("canvas")
            .expect("no canvas element with id 'canvas'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element with id 'canvas' is not a canvas");
        
        let web_options = eframe::WebOptions::default();
        
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(code_analyzer::CodeAnalyzerApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}

mod code_analyzer {
    use eframe::App;
    use egui::{Color32, FontFamily, FontId, Pos2, Rect, Shape, Stroke, Vec2};
    use egui_graphs::{
        DisplayEdge, DisplayNode, DrawContext, EdgeProps, Graph, GraphView, Node, NodeProps,
        SettingsInteraction, SettingsNavigation, SettingsStyle, FruchtermanReingoldState,
        MetadataFrame, get_layout_state, set_layout_state,
    };
    use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
    use petgraph::{stable_graph::{NodeIndex, StableGraph}, Directed, visit::EdgeRef};
    use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

    #[derive(Clone, Debug)]
    pub struct ClassInfo {
        name: String,
        methods: Vec<String>,
        fields: Vec<String>,
        description: String,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum AppTab {
        Graph,
        StressTest,
        Logs,
        NeuralNetwork,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum AttackPattern {
        Flood,
        SlowLoris,
        SynFlood,
        UdpFlood,
        HttpFlood,
    }

    impl AttackPattern {
        fn name(&self) -> &str {
            match self {
                AttackPattern::Flood => "Flood Attack",
                AttackPattern::SlowLoris => "Slowloris",
                AttackPattern::SynFlood => "SYN Flood",
                AttackPattern::UdpFlood => "UDP Flood",
                AttackPattern::HttpFlood => "HTTP Flood",
            }
        }

        fn description(&self) -> &str {
            match self {
                AttackPattern::Flood => "High volume request spam",
                AttackPattern::SlowLoris => "Slow connection exhaustion",
                AttackPattern::SynFlood => "TCP handshake overflow",
                AttackPattern::UdpFlood => "UDP packet bombardment",
                AttackPattern::HttpFlood => "Application layer saturation",
            }
        }

        fn all() -> Vec<AttackPattern> {
            vec![
                AttackPattern::Flood,
                AttackPattern::SlowLoris,
                AttackPattern::SynFlood,
                AttackPattern::UdpFlood,
                AttackPattern::HttpFlood,
            ]
        }
    }

    #[derive(Clone, Debug)]
    struct StressMetrics {
        total_requests: u64,
        successful_requests: u64,
        failed_requests: u64,
        read_operations: u64,
        write_operations: u64,
        bytes_sent: u64,
        bytes_received: u64,
        peak_throughput: f64,
        current_throughput: f64,
        avg_response_time: f64,
        start_time: Option<f64>,
        elapsed_time: f64,
    }

    impl Default for StressMetrics {
        fn default() -> Self {
            Self {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                read_operations: 0,
                write_operations: 0,
                bytes_sent: 0,
                bytes_received: 0,
                peak_throughput: 0.0,
                current_throughput: 0.0,
                avg_response_time: 0.0,
                start_time: None,
                elapsed_time: 0.0,
            }
        }
    }

    #[derive(Clone, Debug)]
    struct LogEntry {
        timestamp: f64,
        level: LogLevel,
        message: String,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(dead_code)]
    enum LogLevel {
        Info,
        Warning,
        Error,
        Critical,
    }

    impl LogLevel {
        fn color(&self) -> Color32 {
            match self {
                LogLevel::Info => Color32::from_rgb(100, 200, 255),
                LogLevel::Warning => Color32::from_rgb(255, 200, 100),
                LogLevel::Error => Color32::from_rgb(255, 100, 100),
                LogLevel::Critical => Color32::from_rgb(255, 50, 50),
            }
        }

        fn prefix(&self) -> &str {
            match self {
                LogLevel::Info => "[INFO]",
                LogLevel::Warning => "[WARN]",
                LogLevel::Error => "[ERROR]",
                LogLevel::Critical => "[CRIT]",
            }
        }
    }

    #[derive(Clone, Debug)]
    struct ThroughputDataPoint {
        time: f64,
        value: f64,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum NeuronType {
        Input,
        Hidden,
        Output,
    }

    #[derive(Clone, Debug)]
    struct NeuronState {
        neuron_type: NeuronType,
        #[allow(dead_code)]
        layer: usize,
        #[allow(dead_code)]
        position_in_layer: usize,
        activation: f32,
        is_firing: bool,
        fire_time: Option<f64>,
        #[allow(dead_code)]
        fire_duration: f32,
    }

    impl Default for NeuronState {
        fn default() -> Self {
            Self {
                neuron_type: NeuronType::Hidden,
                layer: 0,
                position_in_layer: 0,
                activation: 0.0,
                is_firing: false,
                fire_time: None,
                fire_duration: 0.3,
            }
        }
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct NeuralNetworkConfig {
        input_layers: usize,
        hidden_layers: usize,
        output_layers: usize,
        neurons_per_input_layer: usize,
        neurons_per_hidden_layer: usize,
        neurons_per_output_layer: usize,
        fire_rate: f32,
        fire_propagation_speed: f32,
        neuron_inactive_color: [u8; 3],
        neuron_firing_color: [u8; 3],
        input_neuron_color: [u8; 3],
        output_neuron_color: [u8; 3],
        synapse_color: [u8; 3],
        synapse_active_color: [u8; 3],
        show_neuron_values: bool,
    }

    impl Default for NeuralNetworkConfig {
        fn default() -> Self {
            Self {
                input_layers: 1,
                hidden_layers: 2,
                output_layers: 1,
                neurons_per_input_layer: 3,
                neurons_per_hidden_layer: 5,
                neurons_per_output_layer: 2,
                fire_rate: 2.0,
                fire_propagation_speed: 0.5,
                neuron_inactive_color: [100, 100, 150],
                neuron_firing_color: [255, 200, 50],
                input_neuron_color: [100, 200, 100],
                output_neuron_color: [200, 100, 200],
                synapse_color: [80, 80, 80],
                synapse_active_color: [255, 150, 0],
                show_neuron_values: true,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum Relationship {
        OneToOne,
        OneToMany,
        ManyToMany,
    }

    impl Relationship {
        fn label(&self) -> &str {
            match self {
                Relationship::OneToOne => "1:1",
                Relationship::OneToMany => "1:N",
                Relationship::ManyToMany => "N:N",
            }
        }
    }

    #[derive(Clone, Debug)]
    struct CodeNode {
        pos: Pos2,
        z_pos: f32,
        selected: bool,
        dragged: bool,
        hovered: bool,
        class_name: String,
        radius: f32,
        node_color: [u8; 3],
        hover_color: [u8; 3],
        selected_color: [u8; 3],
        use_sphere_rendering: bool,
    }

    impl From<NodeProps<ClassInfo>> for CodeNode {
        fn from(node_props: NodeProps<ClassInfo>) -> Self {
            let default_config = AppConfig::default();
            Self {
                pos: node_props.location(),
                z_pos: 0.0,
                selected: node_props.selected,
                dragged: node_props.dragged,
                hovered: node_props.hovered,
                class_name: node_props.payload.name.clone(),
                radius: 30.0,
                node_color: default_config.node_color,
                hover_color: default_config.node_hover_color,
                selected_color: default_config.node_selected_color,
                use_sphere_rendering: false,
            }
        }
    }

    impl CodeNode {
        fn set_class_name(&mut self, name: String) {
            self.class_name = name;
        }
    }

    impl DisplayNode<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx> for CodeNode {
        fn is_inside(&self, pos: Pos2) -> bool {
            let dir = pos - self.pos;
            dir.length() <= self.radius
        }

        fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
            self.pos + dir.normalized() * self.radius
        }

        fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
            let mut shapes = Vec::new();
            let screen_pos = ctx.meta.canvas_to_screen_pos(self.pos);
            let screen_radius = ctx.meta.canvas_to_screen_size(self.radius);
            
            // Note: depth_scale_strength and depth_fade_strength are applied in the update loop
            // where we have access to config. Here we use the pre-calculated z_pos effects.
            let adjusted_radius = screen_radius;

            let color = if self.selected {
                Color32::from_rgb(self.selected_color[0], self.selected_color[1], self.selected_color[2])
            } else if self.hovered {
                Color32::from_rgb(self.hover_color[0], self.hover_color[1], self.hover_color[2])
            } else {
                Color32::from_rgb(self.node_color[0], self.node_color[1], self.node_color[2])
            };
            
            // Color is adjusted in the update method based on config
            let adjusted_color = color;

            let stroke = if self.selected {
                Stroke::new(2.0, Color32::WHITE)
            } else {
                Stroke::new(1.0, Color32::GRAY)
            };

            if self.use_sphere_rendering {
                // Render as a shaded sphere with gradient
                let light_dir_raw = Vec2::new(-0.5, -0.7);
                let light_len = (light_dir_raw.x * light_dir_raw.x + light_dir_raw.y * light_dir_raw.y).sqrt();
                let light_dir = light_dir_raw / light_len; // Normalized light from top-left
                let num_layers = 8;
                
                for i in 0..num_layers {
                    let t = (i as f32) / (num_layers as f32);
                    let layer_radius = adjusted_radius * (1.0 - t * 0.6);
                    
                    // Calculate lighting based on distance from center
                    let brightness = 1.0 - t * 0.7;
                    let shaded_color = Color32::from_rgb(
                        (adjusted_color.r() as f32 * brightness) as u8,
                        (adjusted_color.g() as f32 * brightness) as u8,
                        (adjusted_color.b() as f32 * brightness) as u8,
                    );
                    
                    // Offset the highlight based on light direction
                    let offset = light_dir * adjusted_radius * 0.2 * (1.0 - t);
                    let layer_center = screen_pos + offset;
                    
                    shapes.push(
                        egui::epaint::CircleShape {
                            center: layer_center,
                            radius: layer_radius,
                            fill: shaded_color,
                            stroke: Stroke::NONE,
                        }
                        .into(),
                    );
                }
                
                // Add rim highlight
                if self.selected {
                    shapes.push(
                        egui::epaint::CircleShape {
                            center: screen_pos,
                            radius: adjusted_radius,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(2.0, Color32::WHITE),
                        }
                        .into(),
                    );
                }
            } else {
                // Flat circle rendering
                shapes.push(
                    egui::epaint::CircleShape {
                        center: screen_pos,
                        radius: adjusted_radius,
                        fill: adjusted_color,
                        stroke,
                    }
                    .into(),
                );
            }

            let font_size = (adjusted_radius * 0.4).max(8.0).min(16.0);
            let galley = ctx.ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    self.class_name.clone(),
                    FontId::new(font_size, FontFamily::Proportional),
                    Color32::WHITE,
                )
            });

            let text_pos = Pos2::new(
                screen_pos.x - galley.size().x / 2.0,
                screen_pos.y - galley.size().y / 2.0,
            );

            shapes.push(egui::epaint::TextShape::new(text_pos, galley, Color32::WHITE).into());
            shapes
        }

        fn update(&mut self, state: &NodeProps<ClassInfo>) {
            self.pos = state.location();
            self.selected = state.selected;
            self.dragged = state.dragged;
            self.hovered = state.hovered;
            self.class_name = state.payload.name.clone();
        }
    }

    #[derive(Clone, Debug)]
    struct CodeEdge {
        order: usize,
        selected: bool,
        label: String,
        edge_color: [u8; 3],
        selected_color: [u8; 3],
    }

    impl From<EdgeProps<Relationship>> for CodeEdge {
        fn from(edge_props: EdgeProps<Relationship>) -> Self {
            let default_config = AppConfig::default();
            Self {
                order: edge_props.order,
                selected: edge_props.selected,
                label: edge_props.payload.label().to_string(),
                edge_color: default_config.edge_color,
                selected_color: default_config.edge_selected_color,
            }
        }
    }

    impl DisplayEdge<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>
        for CodeEdge
    {
        fn is_inside(
            &self,
            start: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            end: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            pos: Pos2,
        ) -> bool {
            let start_pos = start.location();
            let end_pos = end.location();
            let radius = 5.0;
            let line_vec = end_pos - start_pos;
            let point_vec = pos - start_pos;
            let line_len = line_vec.length();
            if line_len < 0.001 {
                return false;
            }
            let proj = point_vec.dot(line_vec) / line_len;
            if proj < 0.0 || proj > line_len {
                return false;
            }
            let closest = start_pos + line_vec.normalized() * proj;
            (pos - closest).length() <= radius
        }

        fn shapes(
            &mut self,
            start: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            end: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            ctx: &DrawContext,
        ) -> Vec<Shape> {
            let start_pos = start.location();
            let end_pos = end.location();

            let dir = (end_pos - start_pos).normalized();
            let start_boundary = start.display().closest_boundary_point(dir);
            let end_boundary = end.display().closest_boundary_point(-dir);
            let mut shapes = Vec::new();
            let screen_start = ctx.meta.canvas_to_screen_pos(start_boundary);
            let screen_end = ctx.meta.canvas_to_screen_pos(end_boundary);

            let color = if self.selected {
                Color32::from_rgb(self.selected_color[0], self.selected_color[1], self.selected_color[2])
            } else {
                Color32::from_rgb(self.edge_color[0], self.edge_color[1], self.edge_color[2])
            };
            let stroke = Stroke::new(2.0, color);

            shapes.push(egui::epaint::Shape::line_segment([screen_start, screen_end], stroke));

            let dir = (screen_end - screen_start).normalized();
            let arrow_size = 10.0;
            let perp = Vec2::new(-dir.y, dir.x);
            let tip = screen_end - dir * arrow_size;
            let left = tip + perp * arrow_size * 0.5;
            let right = tip - perp * arrow_size * 0.5;

            shapes.push(egui::epaint::Shape::convex_polygon(
                vec![screen_end, left, right],
                color,
                Stroke::NONE,
            ));

            let midpoint = (screen_start + screen_end.to_vec2()) * 0.5;
            let galley = ctx.ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    self.label.clone(),
                    FontId::new(12.0, FontFamily::Proportional),
                    color,
                )
            });

            let label_pos = Pos2::new(
                midpoint.x - galley.size().x / 2.0,
                midpoint.y - galley.size().y / 2.0 - 10.0,
            );

            let label_rect = Rect::from_min_size(label_pos, galley.size() + Vec2::new(4.0, 2.0));
            shapes.push(egui::epaint::Shape::rect_filled(
                label_rect,
                2.0,
                Color32::from_black_alpha(200),
            ));

            shapes.push(
                egui::epaint::TextShape::new(label_pos + Vec2::new(2.0, 1.0), galley, Color32::WHITE).into(),
            );

            shapes
        }

        fn update(&mut self, state: &EdgeProps<Relationship>) {
            self.order = state.order;
            self.selected = state.selected;
            self.label = state.payload.label().to_string();
        }
    }

    // Neural Network Display Nodes and Edges
    #[derive(Clone, Debug)]
    struct NeuronNode {
        pos: Pos2,
        selected: bool,
        dragged: bool,
        hovered: bool,
        neuron_type: NeuronType,
        is_firing: bool,
        activation: f32,
        radius: f32,
        show_values: bool,
    }

    impl From<NodeProps<NeuronState>> for NeuronNode {
        fn from(node_props: NodeProps<NeuronState>) -> Self {
            Self {
                pos: node_props.location(),
                selected: node_props.selected,
                dragged: node_props.dragged,
                hovered: node_props.hovered,
                neuron_type: node_props.payload.neuron_type,
                is_firing: node_props.payload.is_firing,
                activation: node_props.payload.activation,
                radius: 25.0,
                show_values: true,
            }
        }
    }

    impl DisplayNode<NeuronState, f32, Directed, petgraph::graph::DefaultIx> for NeuronNode {
        fn is_inside(&self, pos: Pos2) -> bool {
            let dir = pos - self.pos;
            dir.length() <= self.radius
        }

        fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
            self.pos + dir.normalized() * self.radius
        }

        fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
            let mut shapes = Vec::new();
            let screen_pos = ctx.meta.canvas_to_screen_pos(self.pos);
            let screen_radius = ctx.meta.canvas_to_screen_size(self.radius);

            // Get colors from context (we'll pass config through update)
            let color = if self.is_firing {
                Color32::from_rgb(255, 200, 50)
            } else {
                match self.neuron_type {
                    NeuronType::Input => Color32::from_rgb(100, 200, 100),
                    NeuronType::Hidden => Color32::from_rgb(100, 100, 150),
                    NeuronType::Output => Color32::from_rgb(200, 100, 200),
                }
            };

            let border_color = if self.selected {
                Color32::from_rgb(255, 255, 255)
            } else if self.hovered {
                Color32::from_rgb(200, 200, 200)
            } else {
                Color32::from_rgb(80, 80, 80)
            };

            shapes.push(egui::epaint::Shape::circle_filled(screen_pos, screen_radius, color));
            shapes.push(egui::epaint::Shape::circle_stroke(
                screen_pos,
                screen_radius,
                Stroke::new(2.0, border_color),
            ));

            // Draw activation level as inner circle
            if self.activation > 0.1 {
                let activation_radius = screen_radius * self.activation;
                shapes.push(egui::epaint::Shape::circle_filled(
                    screen_pos,
                    activation_radius,
                    Color32::from_rgba_premultiplied(255, 255, 255, 100),
                ));
            }
            
            // Draw activation value text in center (if enabled)
            if self.show_values {
                let activation_text = format!("{:.2}", self.activation);
                let galley = ctx.ctx.fonts_mut(|f| {
                    f.layout_no_wrap(
                        activation_text,
                        FontId::new(10.0, FontFamily::Monospace),
                        Color32::WHITE,
                    )
                });
                
                let text_pos = Pos2::new(
                    screen_pos.x - galley.size().x / 2.0,
                    screen_pos.y - galley.size().y / 2.0,
                );
                
                shapes.push(egui::epaint::Shape::galley(
                    text_pos,
                    galley,
                    Color32::WHITE,
                ));
            }

            shapes
        }

        fn update(&mut self, state: &NodeProps<NeuronState>) {
            self.pos = state.location();
            self.selected = state.selected;
            self.dragged = state.dragged;
            self.hovered = state.hovered;
            self.is_firing = state.payload.is_firing;
            self.activation = state.payload.activation;
        }
    }

    #[derive(Clone, Debug)]
    struct SynapseEdge {
        order: usize,
        selected: bool,
        weight: f32,
        is_active: bool,
    }

    impl From<EdgeProps<f32>> for SynapseEdge {
        fn from(edge_props: EdgeProps<f32>) -> Self {
            Self {
                order: edge_props.order,
                selected: edge_props.selected,
                weight: edge_props.payload,
                is_active: false,
            }
        }
    }

    impl DisplayEdge<NeuronState, f32, Directed, petgraph::graph::DefaultIx, NeuronNode>
        for SynapseEdge
    {
        fn is_inside(
            &self,
            start: &Node<NeuronState, f32, Directed, petgraph::graph::DefaultIx, NeuronNode>,
            end: &Node<NeuronState, f32, Directed, petgraph::graph::DefaultIx, NeuronNode>,
            pos: Pos2,
        ) -> bool {
            let start_pos = start.location();
            let end_pos = end.location();
            let radius = 3.0;
            let line_vec = end_pos - start_pos;
            let point_vec = pos - start_pos;
            let line_len = line_vec.length();
            if line_len < 0.001 {
                return false;
            }
            let proj = point_vec.dot(line_vec) / line_len;
            if proj < 0.0 || proj > line_len {
                return false;
            }
            let closest = start_pos + line_vec.normalized() * proj;
            (pos - closest).length() <= radius
        }

        fn shapes(
            &mut self,
            start: &Node<NeuronState, f32, Directed, petgraph::graph::DefaultIx, NeuronNode>,
            end: &Node<NeuronState, f32, Directed, petgraph::graph::DefaultIx, NeuronNode>,
            ctx: &DrawContext,
        ) -> Vec<Shape> {
            let start_pos = start.location();
            let end_pos = end.location();

            let dir = (end_pos - start_pos).normalized();
            let start_boundary = start.display().closest_boundary_point(dir);
            let end_boundary = end.display().closest_boundary_point(-dir);
            
            let screen_start = ctx.meta.canvas_to_screen_pos(start_boundary);
            let screen_end = ctx.meta.canvas_to_screen_pos(end_boundary);

            let is_source_firing = start.payload().is_firing;
            let color = if is_source_firing || self.is_active {
                Color32::from_rgb(255, 150, 0)
            } else {
                let alpha = (self.weight.abs() * 255.0).min(255.0) as u8;
                Color32::from_rgba_premultiplied(80, 80, 80, alpha)
            };

            let width = if is_source_firing { 3.0 } else { 1.5 };
            let stroke = Stroke::new(width, color);

            vec![egui::epaint::Shape::line_segment([screen_start, screen_end], stroke)]
        }

        fn update(&mut self, state: &EdgeProps<f32>) {
            self.order = state.order;
            self.selected = state.selected;
            self.weight = state.payload;
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    enum VisualizationMode {
        TwoD,
        ThreeD,
    }

    impl VisualizationMode {
        fn label(&self) -> &str {
            match self {
                VisualizationMode::TwoD => "2D View",
                VisualizationMode::ThreeD => "3D View",
            }
        }
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    #[serde(default)]
    struct AppConfig {
        // Visualization mode
        visualization_mode: VisualizationMode,
        
        // 3D settings - Rotation
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
        auto_rotate: bool,
        rotation_speed: f32,
        
        // 3D settings - Positioning
        z_spacing: f32,
        layer_offset: f32,
        perspective_strength: f32,
        
        // 3D settings - Visual Effects
        depth_fade_enabled: bool,
        depth_fade_strength: f32,
        depth_scale_enabled: bool,
        depth_scale_strength: f32,
        use_sphere_rendering: bool,
        
        // Interaction settings
        dragging_enabled: bool,
        hover_enabled: bool,
        node_clicking_enabled: bool,
        node_selection_enabled: bool,
        node_selection_multi_enabled: bool,
        edge_clicking_enabled: bool,
        edge_selection_enabled: bool,
        edge_selection_multi_enabled: bool,
        
        // Navigation settings
        fit_to_screen_enabled: bool,
        zoom_and_pan_enabled: bool,
        fit_to_screen_padding: f32,
        zoom_speed: f32,
        
        // Style settings
        labels_always: bool,
        
        // Color settings
        node_color: [u8; 3],
        node_hover_color: [u8; 3],
        node_selected_color: [u8; 3],
        edge_color: [u8; 3],
        edge_selected_color: [u8; 3],
        background_color: [u8; 3],
        
        // Neural network settings
        nn_config: NeuralNetworkConfig,
        
        // Graph generation settings
        num_nodes: usize,
        num_edges: usize,
        
        // Simulation settings
        simulation_running: bool,
        
        // UI settings
        hover_window_size: f32,
        show_side_panel: bool,
        
        // Grid and axes settings
        show_grid: bool,
        show_axes: bool,
        grid_spacing: f32,
        hierarchical_row_spacing: f32,
        hierarchical_column_spacing: f32,
    }

    impl Default for AppConfig {
        fn default() -> Self {
            Self {
                visualization_mode: VisualizationMode::TwoD,
                rotation_x: 30.0,
                rotation_y: 45.0,
                rotation_z: 0.0,
                auto_rotate: false,
                rotation_speed: 0.5,
                z_spacing: 100.0,
                layer_offset: 0.0,
                perspective_strength: 0.001,
                depth_fade_enabled: true,
                depth_fade_strength: 0.0005,
                depth_scale_enabled: true,
                depth_scale_strength: 0.001,
                use_sphere_rendering: false,
                dragging_enabled: true,
                hover_enabled: true,
                node_clicking_enabled: true,
                node_selection_enabled: true,
                node_selection_multi_enabled: false,
                edge_clicking_enabled: false,
                edge_selection_enabled: false,
                edge_selection_multi_enabled: false,
                fit_to_screen_enabled: false,
                zoom_and_pan_enabled: true,
                fit_to_screen_padding: 0.1,
                zoom_speed: 0.1,
                labels_always: true,
                num_nodes: 4,
                num_edges: 4,
                simulation_running: true,
                node_color: [100, 150, 200],
                node_hover_color: [150, 150, 200],
                node_selected_color: [100, 200, 255],
                edge_color: [128, 128, 128],
                edge_selected_color: [255, 200, 100],
                background_color: [30, 30, 30],
                hover_window_size: 0.0625,
                show_side_panel: true,
                nn_config: NeuralNetworkConfig::default(),
                show_grid: true,
                show_axes: true,
                grid_spacing: 100.0,
                hierarchical_row_spacing: 150.0,
                hierarchical_column_spacing: 180.0,
            }
        }
    }

    // Helper function for 3D to 2D projection with all rotation axes
    fn project_3d_to_2d(
        pos: Pos2,
        pivot: Pos2,
        z: f32,
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
        perspective_strength: f32,
    ) -> Pos2 {
        let rx = rotation_x.to_radians();
        let ry = rotation_y.to_radians();
        let rz = rotation_z.to_radians();

        let x0 = pos.x - pivot.x;
        let y0 = pos.y - pivot.y;
        let z0 = z;
        
        // Apply rotation around X axis
        let y1 = y0 * rx.cos() - z0 * rx.sin();
        let z1 = y0 * rx.sin() + z0 * rx.cos();
        
        // Apply rotation around Y axis
        let x2 = x0 * ry.cos() + z1 * ry.sin();
        let z2 = -x0 * ry.sin() + z1 * ry.cos();
        
        // Apply rotation around Z axis
        let x3 = x2 * rz.cos() - y1 * rz.sin();
        let y2 = x2 * rz.sin() + y1 * rz.cos();
        
        // Perspective projection - depth affects scale
        let denom = 1.0 + z2 * perspective_strength;
        let scale = if denom.abs() < 0.000_1 { 1.0 } else { 1.0 / denom };
        let projected_local = Pos2::new(x3 * scale, y2 * scale);
        pivot + projected_local.to_vec2()
    }

    pub struct CodeAnalyzerApp {
        graph: Graph<ClassInfo, Relationship, Directed, u32, CodeNode, CodeEdge>,
        class_details: HashMap<NodeIndex<u32>, ClassInfo>,
        hovered_node: Option<NodeIndex<u32>>,
        config: AppConfig,
        show_config_window: bool,
        config_search: String,
        auto_save_config: bool,
        show_color_picker: bool,
        // Stress test fields
        current_tab: AppTab,
        stress_active: bool,
        stress_metrics: StressMetrics,
        attack_pattern: AttackPattern,
        attack_intensity: f32,
        max_requests_per_sec: u32,
        logs: VecDeque<LogEntry>,
        max_log_entries: usize,
        throughput_history: VecDeque<ThroughputDataPoint>,
        max_throughput_points: usize,
        last_update_time: Option<f64>,
        graph_pivot: Pos2,
        // File operations
        show_file_dialog: bool,
        file_dialog_message: String,
        show_export_dialog: bool,
        export_format: ExportFormat,
        #[allow(dead_code)]
        loaded_file_name: Option<String>,
        // Neural network fields
        neural_network: Option<Graph<NeuronState, f32, Directed, u32, NeuronNode, SynapseEdge>>,
        neuron_states: HashMap<NodeIndex<u32>, NeuronState>,
        nn_last_fire_time: f64,
        // Fit to screen tracking
        fit_to_screen_counter: u32,
        // Node text editing
        editing_node: Option<NodeIndex<u32>>,
        edit_text: String,
        // Popup window sizes per node
        popup_sizes: HashMap<NodeIndex<u32>, Vec2>,
        // Markdown cache for rendering
        markdown_cache: CommonMarkCache,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum ExportFormat {
        Json,
        Csv,
        Graphviz,
    }

    impl ExportFormat {
        #[allow(dead_code)]
        fn name(&self) -> &str {
            match self {
                ExportFormat::Json => "JSON",
                ExportFormat::Csv => "CSV",
                ExportFormat::Graphviz => "Graphviz DOT",
            }
        }

        fn extension(&self) -> &str {
            match self {
                ExportFormat::Json => "json",
                ExportFormat::Csv => "csv",
                ExportFormat::Graphviz => "dot",
            }
        }
    }

    const GRAPH_VIEW_ID: &str = "code_analyzer_graph";
    const NEURAL_VIEW_ID: &str = "code_analyzer_neural";

    impl CodeAnalyzerApp {
        pub fn new(cc: &eframe::CreationContext) -> Self {
            // Load config from storage if available
            let mut config: AppConfig = cc
                .storage
                .and_then(|s| eframe::get_value(s, "app_config"))
                .unwrap_or_default();
            
            let auto_save = cc
                .storage
                .and_then(|s| eframe::get_value(s, "auto_save_config"))
                .unwrap_or(true);
            
            config.fit_to_screen_enabled = true; // Center graph on initial load
            
            let mut pg = StableGraph::<ClassInfo, Relationship, Directed>::new();
            let mut class_details = HashMap::new();

            let user = pg.add_node(ClassInfo {
                name: "User".to_string(),
                methods: vec![
                    "login()".to_string(),
                    "logout()".to_string(),
                    "updateProfile()".to_string(),
                ],
                fields: vec![
                    "id: int".to_string(),
                    "username: string".to_string(),
                    "email: string".to_string(),
                ],
                description: String::new(),
            });

            let order = pg.add_node(ClassInfo {
                name: "Order".to_string(),
                methods: vec![
                    "calculate()".to_string(),
                    "submit()".to_string(),
                    "cancel()".to_string(),
                ],
                fields: vec![
                    "id: int".to_string(),
                    "total: float".to_string(),
                    "status: string".to_string(),
                ],
                description: String::new(),
            });

            let product = pg.add_node(ClassInfo {
                name: "Product".to_string(),
                methods: vec!["getPrice()".to_string(), "updateStock()".to_string()],
                fields: vec![
                    "id: int".to_string(),
                    "name: string".to_string(),
                    "price: float".to_string(),
                ],
                description: String::new(),
            });

            let payment = pg.add_node(ClassInfo {
                name: "Payment".to_string(),
                methods: vec!["process()".to_string(), "refund()".to_string()],
                fields: vec![
                    "id: int".to_string(),
                    "amount: float".to_string(),
                    "method: string".to_string(),
                ],
                description: String::new(),
            });

            for idx in pg.node_indices() {
                if let Some(node) = pg.node_weight(idx) {
                    class_details.insert(idx, node.clone());
                }
            }

            pg.add_edge(user, order, Relationship::OneToMany);
            pg.add_edge(order, product, Relationship::ManyToMany);
            pg.add_edge(order, payment, Relationship::OneToOne);
            pg.add_edge(user, payment, Relationship::OneToMany);

            let mut graph = Graph::<ClassInfo, Relationship, Directed, u32, CodeNode, CodeEdge>::from(&pg);

            let positions = vec![
                Pos2::new(100.0, 100.0),
                Pos2::new(300.0, 50.0),
                Pos2::new(300.0, 150.0),
                Pos2::new(500.0, 100.0),
            ];

            for (idx, pos) in pg.node_indices().zip(positions.iter()) {
                if let Some(node) = graph.node_mut(idx) {
                    node.set_location(*pos);
                }
            }
            
            Self {
                graph,
                class_details,
                hovered_node: None,
                config,
                show_config_window: false,
                config_search: String::new(),
                auto_save_config: auto_save,
                show_color_picker: false,
                current_tab: AppTab::Graph,
                stress_active: false,
                stress_metrics: StressMetrics::default(),
                attack_pattern: AttackPattern::Flood,
                attack_intensity: 50.0,
                max_requests_per_sec: 1000,
                logs: VecDeque::new(),
                max_log_entries: 1000,
                throughput_history: VecDeque::new(),
                max_throughput_points: 100,
                last_update_time: None,
                graph_pivot: Pos2::ZERO,
                show_file_dialog: false,
                file_dialog_message: String::new(),
                loaded_file_name: None,
                show_export_dialog: false,
                export_format: ExportFormat::Json,
                neural_network: None,
                neuron_states: HashMap::new(),
                nn_last_fire_time: 0.0,
                fit_to_screen_counter: 3, // Start with counter for initial centering
                editing_node: None,
                edit_text: String::new(),
                popup_sizes: HashMap::new(),
                markdown_cache: CommonMarkCache::default(),
            }
        }

        fn regenerate_graph(&mut self, num_nodes: usize, num_edges: usize) {
            let mut pg = StableGraph::<ClassInfo, Relationship, Directed>::new();
            self.class_details.clear();

            // Dictionary with synthetic data for nodes
            let node_names = vec![
                "User", "Order", "Product", "Payment", "Invoice", "Shipment",
                "Customer", "Supplier", "Category", "Review", "Cart", "Warehouse",
                "Address", "Discount", "Notification", "Log", "Account", "Transaction",
                "Profile", "Setting", "Report", "Dashboard", "Analytics", "Session",
                "Token", "Permission", "Role", "Group", "Team", "Department",
                "Project", "Task", "Milestone", "Document", "File", "Folder",
                "Message", "Comment", "Thread", "Channel", "Event", "Calendar",
                "Contact", "Lead", "Opportunity", "Deal", "Quote", "Contract",
                "Subscription", "License", "Plan", "Feature", "Module", "Component",
                "Service", "API", "Endpoint", "Request", "Response", "Error",
                "Database", "Table", "Schema", "Query", "Index", "Backup",
                "Server", "Node", "Cluster", "Network", "LoadBalancer", "Gateway",
                "Security", "Auth", "OAuth", "JWT", "Certificate", "Key",
                "Encryption", "Hash", "Salt", "Signature", "Verification", "Audit",
                "Monitor", "Alert", "Metric", "HealthCheck", "Status", "Uptime",
                "Cache", "Redis", "Queue", "Job", "Worker", "Scheduler",
                "Email", "SMS", "Push", "Webhook", "Integration", "Plugin"
            ];
            
            let mut node_indices = Vec::new();
            for i in 0..num_nodes.max(1) {
                let name_idx = i % node_names.len();
                let node_name = if i < node_names.len() {
                    node_names[name_idx].to_string()
                } else {
                    format!("{}{}", node_names[name_idx], i / node_names.len())
                };
                let node_idx = pg.add_node(ClassInfo {
                    name: node_name.clone(),
                    methods: vec![
                        format!("get{}()", node_name),
                        format!("set{}()", node_name),
                        "update()".to_string(),
                    ],
                    fields: vec![
                        "id: int".to_string(),
                        format!("name: string"),
                        format!("timestamp: datetime"),
                    ],
                    description: String::new(),
                });
                node_indices.push(node_idx);
                
                if let Some(node) = pg.node_weight(node_idx) {
                    self.class_details.insert(node_idx, node.clone());
                }
            }

            // Generate edges with varied relationships (no limits)
            let relationships = [Relationship::OneToOne, Relationship::OneToMany, Relationship::ManyToMany];
            let actual_edges = num_edges;
            
            for i in 0..actual_edges {
                let from_idx = i % node_indices.len();
                let to_idx = (i + 1 + i / node_indices.len()) % node_indices.len();
                
                if from_idx != to_idx {
                    let rel = relationships[i % relationships.len()];
                    pg.add_edge(node_indices[from_idx], node_indices[to_idx], rel);
                }
            }

            // Create new graph from petgraph
            self.graph = Graph::<ClassInfo, Relationship, Directed, u32, CodeNode, CodeEdge>::from(&pg);

            // Set initial random positions
            let radius = 200.0;
            for (i, idx) in node_indices.iter().enumerate() {
                if let Some(node) = self.graph.node_mut(*idx) {
                    let angle = (i as f32) * std::f32::consts::TAU / (node_indices.len() as f32);
                    let pos = Pos2::new(
                        radius * angle.cos(),
                        radius * angle.sin()
                    );
                    node.set_location(pos);
                }
            }
        }

        fn add_log(&mut self, level: LogLevel, message: String, current_time: f64) {
            let entry = LogEntry {
                timestamp: current_time,
                level,
                message,
            };
            
            self.logs.push_front(entry);
            
            if self.logs.len() > self.max_log_entries {
                self.logs.pop_back();
            }
        }

        fn generate_neural_network(&mut self) {
            let nn_cfg = &self.config.nn_config;
            let mut pg = StableGraph::<NeuronState, f32, Directed>::new();
            let mut neuron_states = HashMap::new();
            
            let total_layers = nn_cfg.input_layers + nn_cfg.hidden_layers + nn_cfg.output_layers;
            let mut layer_nodes: Vec<Vec<NodeIndex<u32>>> = vec![Vec::new(); total_layers];
            
            let mut current_layer = 0;
            
            // Create input layers
            for _layer_idx in 0..nn_cfg.input_layers {
                for pos in 0..nn_cfg.neurons_per_input_layer {
                    let state = NeuronState {
                        neuron_type: NeuronType::Input,
                        layer: current_layer,
                        position_in_layer: pos,
                        activation: 0.0,
                        is_firing: false,
                        fire_time: None,
                        fire_duration: 0.3,
                    };
                    let idx = pg.add_node(state.clone());
                    neuron_states.insert(idx, state);
                    layer_nodes[current_layer].push(idx);
                }
                current_layer += 1;
            }
            
            // Create hidden layers
            for _layer_idx in 0..nn_cfg.hidden_layers {
                for pos in 0..nn_cfg.neurons_per_hidden_layer {
                    let state = NeuronState {
                        neuron_type: NeuronType::Hidden,
                        layer: current_layer,
                        position_in_layer: pos,
                        activation: 0.0,
                        is_firing: false,
                        fire_time: None,
                        fire_duration: 0.3,
                    };
                    let idx = pg.add_node(state.clone());
                    neuron_states.insert(idx, state);
                    layer_nodes[current_layer].push(idx);
                }
                current_layer += 1;
            }
            
            // Create output layers
            for _layer_idx in 0..nn_cfg.output_layers {
                for pos in 0..nn_cfg.neurons_per_output_layer {
                    let state = NeuronState {
                        neuron_type: NeuronType::Output,
                        layer: current_layer,
                        position_in_layer: pos,
                        activation: 0.0,
                        is_firing: false,
                        fire_time: None,
                        fire_duration: 0.3,
                    };
                    let idx = pg.add_node(state.clone());
                    neuron_states.insert(idx, state);
                    layer_nodes[current_layer].push(idx);
                }
                current_layer += 1;
            }
            
            // Create synaptic connections between adjacent layers
            for layer_idx in 0..(total_layers - 1) {
                let current_layer_nodes = &layer_nodes[layer_idx];
                let next_layer_nodes = &layer_nodes[layer_idx + 1];
                
                for &source in current_layer_nodes {
                    for &target in next_layer_nodes {
                        // Random weight between -1.0 and 1.0
                        let weight = (js_sys::Math::random() as f32) * 2.0 - 1.0;
                        pg.add_edge(source, target, weight);
                    }
                }
            }
            
            // Create graph and apply neural network layout
            let mut graph = Graph::<NeuronState, f32, Directed, u32, NeuronNode, SynapseEdge>::from(&pg);
            self.layout_neural_network(&mut graph, &layer_nodes);
            
            self.neural_network = Some(graph);
            self.neuron_states = neuron_states;
        }
        
        fn layout_neural_network(&self, graph: &mut Graph<NeuronState, f32, Directed, u32, NeuronNode, SynapseEdge>, layer_nodes: &[Vec<NodeIndex<u32>>]) {
            let total_layers = layer_nodes.len();
            if total_layers == 0 {
                return;
            }
            
            // Calculate spacing
            let horizontal_spacing = 300.0;
            let base_x = -(total_layers as f32 * horizontal_spacing) / 2.0;
            
            for (layer_idx, nodes) in layer_nodes.iter().enumerate() {
                let nodes_in_layer = nodes.len();
                if nodes_in_layer == 0 {
                    continue;
                }
                
                let vertical_spacing = if nodes_in_layer == 1 { 0.0 } else { 400.0 / (nodes_in_layer - 1) as f32 };
                let base_y = -(nodes_in_layer as f32 * vertical_spacing) / 2.0;
                
                let x = base_x + (layer_idx as f32 * horizontal_spacing);
                
                for (pos_idx, &node_idx) in nodes.iter().enumerate() {
                    let y = base_y + (pos_idx as f32 * vertical_spacing);
                    if let Some(node) = graph.node_mut(node_idx) {
                        node.set_location(Pos2::new(x, y));
                    }
                }
            }
        }
        
        fn simulate_neural_network(&mut self, ctx: &egui::Context) {
            if self.neural_network.is_none() {
                return;
            }
            
            let current_time = ctx.input(|i| i.time);
            let fire_rate = self.config.nn_config.fire_rate;
            let fire_duration = 0.3;
            
            // Randomly fire input neurons
            if current_time - self.nn_last_fire_time > 1.0 / fire_rate as f64 {
                self.nn_last_fire_time = current_time;
                
                // Fire a random input neuron
                let input_neurons: Vec<NodeIndex<u32>> = self.neuron_states
                    .iter()
                    .filter(|(_, state)| state.neuron_type == NeuronType::Input)
                    .map(|(idx, _)| *idx)
                    .collect();
                
                if !input_neurons.is_empty() {
                    let random_idx = (js_sys::Math::random() * input_neurons.len() as f64) as usize % input_neurons.len();
                    let neuron_idx = input_neurons[random_idx];
                    
                    if let Some(state) = self.neuron_states.get_mut(&neuron_idx) {
                        state.is_firing = true;
                        state.fire_time = Some(current_time);
                        state.activation = 1.0;
                    }
                }
            }
            
            // Update neuron states and propagate signals
            let mut neurons_to_update = Vec::new();
            
            for (idx, state) in self.neuron_states.iter() {
                if state.is_firing {
                    if let Some(fire_time) = state.fire_time {
                        // Check if fire duration has elapsed
                        if current_time - fire_time > fire_duration as f64 {
                            neurons_to_update.push((*idx, false, 0.0));
                        } else {
                            // Propagate signal to connected neurons
                            if let Some(ref graph) = self.neural_network {
                                let g = graph.g();
                                for edge in g.edges(*idx) {
                                    let target = edge.target();
                                    let weight = *edge.weight().payload();
                                    
                                    // Activate target neuron based on weight
                                    if let Some(target_state) = self.neuron_states.get(&target) {
                                        if !target_state.is_firing {
                                            let activation = (weight.abs() * state.activation).min(1.0);
                                            if activation > 0.5 {
                                                neurons_to_update.push((target, true, activation));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Apply updates
            for (idx, is_firing, activation) in neurons_to_update {
                if let Some(state) = self.neuron_states.get_mut(&idx) {
                    if is_firing && !state.is_firing {
                        state.is_firing = true;
                        state.fire_time = Some(current_time);
                        state.activation = activation;
                    } else if !is_firing {
                        state.is_firing = false;
                        state.fire_time = None;
                        state.activation = 0.0;
                    }
                }
            }
            
            // Update graph node states
            if let Some(ref mut graph) = self.neural_network {
                for (idx, state) in &self.neuron_states {
                    if let Some(node) = graph.node_mut(*idx) {
                        *node.payload_mut() = state.clone();
                    }
                }
            }
        }

        fn simulate_stress_test(&mut self, ctx: &egui::Context) {
            if !self.stress_active {
                return;
            }

            let current_time = ctx.input(|i| i.time);
            
            // Initialize start time
            if self.stress_metrics.start_time.is_none() {
                self.stress_metrics.start_time = Some(current_time);
                self.last_update_time = Some(current_time);
                self.add_log(LogLevel::Info, format!("🚨 Stress test started: {} at intensity {:.0}%", 
                    self.attack_pattern.name(), self.attack_intensity), current_time);
            }

            let start_time = self.stress_metrics.start_time.unwrap();
            self.stress_metrics.elapsed_time = current_time - start_time;

            // Calculate requests per frame based on pattern and intensity
            let base_rps = (self.max_requests_per_sec as f32 * (self.attack_intensity / 100.0)) as u64;
            let delta_time = current_time - self.last_update_time.unwrap_or(current_time);
            
            if delta_time <= 0.0 {
                return;
            }

            let requests_this_frame = match self.attack_pattern {
                AttackPattern::Flood => {
                    // Continuous high volume
                    (base_rps as f64 * delta_time) as u64
                },
                AttackPattern::SlowLoris => {
                    // Many slow connections (lower request rate but sustained)
                    ((base_rps / 2) as f64 * delta_time) as u64
                },
                AttackPattern::SynFlood => {
                    // Burst pattern
                    let cycle = (current_time % 2.0) / 2.0;
                    if cycle < 0.3 {
                        (base_rps as f64 * delta_time * 3.0) as u64
                    } else {
                        (base_rps as f64 * delta_time * 0.5) as u64
                    }
                },
                AttackPattern::UdpFlood => {
                    // Very high volume, lower reliability
                    (base_rps as f64 * delta_time * 1.5) as u64
                },
                AttackPattern::HttpFlood => {
                    // Application layer - moderate volume
                    ((base_rps as f64 * 0.8) * delta_time) as u64
                },
            };

            // Simulate request processing
            let success_rate = match self.attack_pattern {
                AttackPattern::Flood => 0.85,
                AttackPattern::SlowLoris => 0.70,
                AttackPattern::SynFlood => 0.60,
                AttackPattern::UdpFlood => 0.50,
                AttackPattern::HttpFlood => 0.75,
            };

            let successful = (requests_this_frame as f64 * success_rate) as u64;
            let failed = requests_this_frame - successful;

            self.stress_metrics.total_requests += requests_this_frame;
            self.stress_metrics.successful_requests += successful;
            self.stress_metrics.failed_requests += failed;

            // Simulate read/write operations
            let read_ops = (requests_this_frame as f64 * 1.5) as u64;
            let write_ops = (requests_this_frame as f64 * 0.3) as u64;
            self.stress_metrics.read_operations += read_ops;
            self.stress_metrics.write_operations += write_ops;

            // Simulate bandwidth (bytes)
            let avg_request_size = 512; // bytes
            let avg_response_size = 2048; // bytes
            self.stress_metrics.bytes_sent += requests_this_frame * avg_request_size;
            self.stress_metrics.bytes_received += successful * avg_response_size;

            // Calculate throughput (MB/s)
            let throughput_bytes = (self.stress_metrics.bytes_sent + self.stress_metrics.bytes_received) as f64;
            self.stress_metrics.current_throughput = (throughput_bytes / self.stress_metrics.elapsed_time) / (1024.0 * 1024.0);
            
            if self.stress_metrics.current_throughput > self.stress_metrics.peak_throughput {
                self.stress_metrics.peak_throughput = self.stress_metrics.current_throughput;
            }

            // Update avg response time (simulated)
            let base_response_time = match self.attack_pattern {
                AttackPattern::Flood => 150.0,
                AttackPattern::SlowLoris => 5000.0,
                AttackPattern::SynFlood => 200.0,
                AttackPattern::UdpFlood => 100.0,
                AttackPattern::HttpFlood => 300.0,
            };
            let load_factor = (self.stress_metrics.total_requests as f64 / 1000.0).min(10.0);
            self.stress_metrics.avg_response_time = base_response_time * (1.0 + load_factor * 0.1);

            // Add throughput data point
            self.throughput_history.push_back(ThroughputDataPoint {
                time: self.stress_metrics.elapsed_time,
                value: self.stress_metrics.current_throughput,
            });
            
            if self.throughput_history.len() > self.max_throughput_points {
                self.throughput_history.pop_front();
            }

            // Log critical events
            if self.stress_metrics.total_requests % 10000 == 0 && self.stress_metrics.total_requests > 0 {
                self.add_log(LogLevel::Warning, 
                    format!("⚠️ {} requests processed, {:.1}% success rate", 
                        self.stress_metrics.total_requests,
                        (self.stress_metrics.successful_requests as f64 / self.stress_metrics.total_requests as f64) * 100.0
                    ), current_time);
            }

            if self.stress_metrics.current_throughput > 100.0 {
                if self.stress_metrics.total_requests % 5000 == 0 {
                    self.add_log(LogLevel::Critical, 
                        format!("🔥 High throughput detected: {:.2} MB/s", self.stress_metrics.current_throughput), 
                        current_time);
                }
            }

            self.last_update_time = Some(current_time);
            ctx.request_repaint();
        }

        fn stop_stress_test(&mut self, current_time: f64) {
            if self.stress_active {
                self.stress_active = false;
                self.add_log(LogLevel::Info, 
                    format!("✅ Stress test stopped. Total requests: {}, Duration: {:.1}s", 
                        self.stress_metrics.total_requests,
                        self.stress_metrics.elapsed_time
                    ), current_time);
            }
        }

        fn reset_stress_metrics(&mut self, current_time: f64) {
            self.stress_metrics = StressMetrics::default();
            self.throughput_history.clear();
            self.last_update_time = None;
            self.add_log(LogLevel::Info, "🔄 Metrics reset".to_string(), current_time);
        }

        fn trigger_file_upload(&mut self) {
            self.file_dialog_message = "File upload from browser is a restricted operation.\nUse 'Load Example Graph' instead.".to_string();
            self.show_file_dialog = true;
        }

        fn export_graph_json(&self) -> String {
            let mut json = String::from("{\n  \"nodes\": [\n");
            
            let node_count = self.class_details.len();
            for (i, (_, info)) in self.class_details.iter().enumerate() {
                json.push_str("    {\n");
                json.push_str(&format!("      \"name\": \"{}\",\n", info.name));
                json.push_str("      \"methods\": [");
                for (j, method) in info.methods.iter().enumerate() {
                    json.push_str(&format!("\"{}\"", method));
                    if j < info.methods.len() - 1 {
                        json.push_str(", ");
                    }
                }
                json.push_str("],\n");
                json.push_str("      \"fields\": [");
                for (j, field) in info.fields.iter().enumerate() {
                    json.push_str(&format!("\"{}\"", field));
                    if j < info.fields.len() - 1 {
                        json.push_str(", ");
                    }
                }
                json.push_str("]\n");
                json.push_str("    }");
                if i < node_count - 1 {
                    json.push_str(",");
                }
                json.push_str("\n");
            }
            
            json.push_str("  ],\n  \"edges\": [\n");
            
            let edges: Vec<_> = self.graph.g().edge_indices().collect();
            for (i, edge_idx) in edges.iter().enumerate() {
                if let Some(edge) = self.graph.g().edge_endpoints(*edge_idx) {
                    if let Some(relationship) = self.graph.g().edge_weight(*edge_idx) {
                        json.push_str(&format!("    {{\"from\": {}, \"to\": {}, \"relationship\": \"{}\"}}", 
                            edge.0.index(), edge.1.index(), relationship.label()));
                        if i < edges.len() - 1 {
                            json.push_str(",");
                        }
                        json.push_str("\n");
                    }
                }
            }
            
            json.push_str("  ]\n}");
            json
        }

        fn export_graph_csv(&self) -> String {
            let mut csv = String::from("Type,Name,Methods,Fields,From,To,Relationship\n");
            
            for (_, info) in &self.class_details {
                csv.push_str(&format!("Node,\"{}\",\"{}\",\"{}\",,,\n",
                    info.name,
                    info.methods.join("; "),
                    info.fields.join("; ")
                ));
            }

            for edge_idx in self.graph.g().edge_indices() {
                if let Some(edge) = self.graph.g().edge_endpoints(edge_idx) {
                    if let Some(relationship) = self.graph.g().edge_weight(edge_idx) {
                        csv.push_str(&format!("Edge,,,,{},{},{}\n",
                            edge.0.index(),
                            edge.1.index(),
                            relationship.label()
                        ));
                    }
                }
            }

            csv
        }

        fn export_graph_graphviz(&self) -> String {
            let mut dot = String::from("digraph G {\n");
            dot.push_str("  rankdir=LR;\n");
            dot.push_str("  node [shape=box];\n\n");

            for (idx, info) in &self.class_details {
                dot.push_str(&format!("  {} [label=\"{}\"];\n", idx.index(), info.name));
            }

            dot.push_str("\n");

            for edge_idx in self.graph.g().edge_indices() {
                if let Some(edge) = self.graph.g().edge_endpoints(edge_idx) {
                    if let Some(relationship) = self.graph.g().edge_weight(edge_idx) {
                        dot.push_str(&format!("  {} -> {} [label=\"{}\"];\n",
                            edge.0.index(),
                            edge.1.index(),
                            relationship.label()
                        ));
                    }
                }
            }

            dot.push_str("}\n");
            dot
        }

        fn download_file(&self, filename: &str, content: &str) {
            use wasm_bindgen::JsCast;
            
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    // Create blob
                    let array = js_sys::Array::new();
                    array.push(&wasm_bindgen::JsValue::from_str(content));
                    
                    if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&array) {
                        // Create download link
                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                            if let Ok(element) = document.create_element("a") {
                                if let Ok(link) = element.dyn_into::<web_sys::HtmlAnchorElement>() {
                                    link.set_href(&url);
                                    link.set_download(filename);
                                    let _ = link.click();
                                    web_sys::Url::revoke_object_url(&url).ok();
                                }
                            }
                        }
                    }
                }
            }
        }

        fn draw_grid_and_axes(
            &self,
            ui: &mut egui::Ui,
            view_id: Option<&'static str>,
            view_rect: Option<egui::Rect>,
        ) {
            if !self.config.show_grid && !self.config.show_axes {
                return;
            }

            let Some(view_id) = view_id else {
                return;
            };

            let paint_rect = view_rect.unwrap_or_else(|| ui.max_rect());
            if paint_rect.width() <= 0.0 || paint_rect.height() <= 0.0 {
                return;
            }

            let painter = ui.painter_at(paint_rect);
            let mut draw_meta = MetadataFrame::new(Some(view_id.to_string())).load(ui);
            draw_meta.pan += paint_rect.left_top().to_vec2();

            let zoom = draw_meta.zoom.max(0.001);
            let pan = draw_meta.pan;

            let base_spacing = self.config.grid_spacing.max(1.0);
            let mut spacing = base_spacing;
            let min_screen_spacing = 24.0;
            let max_screen_spacing = 120.0;
            while spacing * zoom < min_screen_spacing {
                spacing *= 2.0;
                if spacing > 1_000_000.0 {
                    break;
                }
            }
            while spacing * zoom > max_screen_spacing && spacing > base_spacing {
                spacing *= 0.5;
            }

            let fine_spacing = (spacing * 0.25).max(base_spacing);
            let fine_screen_spacing = fine_spacing * zoom;
            let fine_stroke = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 70));
            let coarse_stroke = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 140));
            let axis_color_x = Color32::from_rgb(255, 100, 100);
            let axis_color_y = Color32::from_rgb(100, 255, 100);
            let axis_color_z = Color32::from_rgb(100, 100, 255);
            let origin_color = Color32::from_rgb(255, 255, 0);

            let canvas_min = draw_meta.screen_to_canvas_pos(paint_rect.min);
            let canvas_max = draw_meta.screen_to_canvas_pos(paint_rect.max);

            if self.config.show_grid {
                let draw_grid = |grid_spacing: f32, stroke: egui::Stroke| {
                    let mut x = (canvas_min.x / grid_spacing).floor() * grid_spacing;
                    let max_x = (canvas_max.x / grid_spacing).ceil() * grid_spacing;
                    let mut guard = 0;
                    while x <= max_x && guard < 2048 {
                        let x_screen = x * zoom + pan.x;
                        painter.line_segment(
                            [Pos2::new(x_screen, paint_rect.top()), Pos2::new(x_screen, paint_rect.bottom())],
                            stroke,
                        );
                        x += grid_spacing;
                        guard += 1;
                    }

                    let mut y = (canvas_min.y / grid_spacing).floor() * grid_spacing;
                    let max_y = (canvas_max.y / grid_spacing).ceil() * grid_spacing;
                    guard = 0;
                    while y <= max_y && guard < 2048 {
                        let y_screen = y * zoom + pan.y;
                        painter.line_segment(
                            [Pos2::new(paint_rect.left(), y_screen), Pos2::new(paint_rect.right(), y_screen)],
                            stroke,
                        );
                        y += grid_spacing;
                        guard += 1;
                    }
                };

                if fine_screen_spacing >= 6.0 && fine_spacing < spacing {
                    draw_grid(fine_spacing, fine_stroke);
                }
                draw_grid(spacing, coarse_stroke);
            }

            if self.config.show_axes {
                let pivot = if self.graph.g().node_count() == 0 {
                    Pos2::ZERO
                } else {
                    self.graph_pivot
                };

                let projected_origin_canvas = if self.config.visualization_mode == VisualizationMode::ThreeD {
                    project_3d_to_2d(
                        pivot,
                        pivot,
                        0.0,
                        self.config.rotation_x,
                        self.config.rotation_y,
                        self.config.rotation_z,
                        self.config.perspective_strength,
                    )
                } else {
                    Pos2::ZERO
                };

                let origin_screen = draw_meta.canvas_to_screen_pos(projected_origin_canvas);

                if self.config.visualization_mode == VisualizationMode::ThreeD {
                    let axis_eps = 1e-4;
                    let axis_length = (spacing * 0.75).clamp(40.0, 600.0);
                    let axes = [
                        ("X", axis_color_x, Pos2::new(pivot.x + axis_length, pivot.y), 0.0),
                        ("Y", axis_color_y, Pos2::new(pivot.x, pivot.y + axis_length), 0.0),
                        ("Z", axis_color_z, pivot, axis_length),
                    ];

                    for (label, color, axis_point, axis_z) in axes {
                        let projected = project_3d_to_2d(
                            axis_point,
                            pivot,
                            axis_z,
                            self.config.rotation_x,
                            self.config.rotation_y,
                            self.config.rotation_z,
                            self.config.perspective_strength,
                        );
                        let axis_screen = draw_meta.canvas_to_screen_pos(projected);

                        if (axis_screen - origin_screen).length_sq() <= axis_eps {
                            continue;
                        }

                        painter.line_segment(
                            [origin_screen, axis_screen],
                            egui::Stroke::new(2.0, color),
                        );

                        let mut label_dir = axis_screen - origin_screen;
                        label_dir = label_dir.normalized() * 16.0;
                        painter.text(
                            axis_screen + label_dir,
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(14.0),
                            color,
                        );
                    }
                } else {
                    if origin_screen.y >= paint_rect.top() && origin_screen.y <= paint_rect.bottom() {
                        painter.line_segment(
                            [Pos2::new(paint_rect.left(), origin_screen.y), Pos2::new(paint_rect.right(), origin_screen.y)],
                            egui::Stroke::new(2.0, axis_color_x),
                        );
                    }

                    if origin_screen.x >= paint_rect.left() && origin_screen.x <= paint_rect.right() {
                        painter.line_segment(
                            [Pos2::new(origin_screen.x, paint_rect.top()), Pos2::new(origin_screen.x, paint_rect.bottom())],
                            egui::Stroke::new(2.0, axis_color_y),
                        );
                    }
                }

                if paint_rect.contains(origin_screen) {
                    painter.circle_filled(origin_screen, 4.0, origin_color);

                    if self.config.visualization_mode != VisualizationMode::ThreeD {
                        painter.text(
                            origin_screen + egui::vec2(8.0, -8.0),
                            egui::Align2::LEFT_BOTTOM,
                            "X",
                            egui::FontId::proportional(14.0),
                            axis_color_x,
                        );
                        painter.text(
                            origin_screen + egui::vec2(-8.0, -12.0),
                            egui::Align2::RIGHT_BOTTOM,
                            "Y",
                            egui::FontId::proportional(14.0),
                            axis_color_y,
                        );
                    }
                }
            }
        }

        fn apply_hierarchical_layout(&mut self) {
            use petgraph::Direction::{Incoming, Outgoing};

            let graph_ref = self.graph.g();
            let node_indices: Vec<_> = graph_ref.node_indices().collect();
            if node_indices.is_empty() {
                return;
            }

            let mut indegree: HashMap<NodeIndex<u32>, usize> = HashMap::new();
            for idx in &node_indices {
                let count = graph_ref.neighbors_directed(*idx, Incoming).count();
                indegree.insert(*idx, count);
            }

            let mut depth: HashMap<NodeIndex<u32>, usize> = HashMap::new();
            let mut queue: VecDeque<_> = node_indices
                .iter()
                .copied()
                .filter(|idx| indegree.get(idx).copied().unwrap_or(0) == 0)
                .collect();
            if queue.is_empty() {
                queue = node_indices.iter().copied().collect();
            }

            let mut visited = HashSet::new();
            while let Some(node_idx) = queue.pop_front() {
                let current_layer = *depth.get(&node_idx).unwrap_or(&0);
                visited.insert(node_idx);

                for neighbor in graph_ref.neighbors_directed(node_idx, Outgoing) {
                    let entry = indegree.entry(neighbor).or_insert(0);
                    if *entry > 0 {
                        *entry -= 1;
                    }
                    depth
                        .entry(neighbor)
                        .and_modify(|layer| *layer = (*layer).max(current_layer + 1))
                        .or_insert(current_layer + 1);
                    if *entry == 0 && !visited.contains(&neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }

            let mut max_depth = depth.values().copied().max().unwrap_or(0);
            for idx in &node_indices {
                depth.entry(*idx).or_insert_with(|| {
                    max_depth += 1;
                    max_depth
                });
            }

            let mut layers: BTreeMap<usize, Vec<NodeIndex<u32>>> = BTreeMap::new();
            for idx in &node_indices {
                let layer = depth.get(idx).copied().unwrap_or(0);
                layers.entry(layer).or_default().push(*idx);
            }

            let row_spacing = self.config.hierarchical_row_spacing.max(10.0);
            let col_spacing = self.config.hierarchical_column_spacing.max(40.0);

            for (layer, mut nodes) in layers {
                nodes.sort_unstable_by_key(|idx| idx.index());
                let width = if nodes.len() > 1 {
                    (nodes.len() - 1) as f32 * col_spacing
                } else {
                    0.0
                };
                let y = layer as f32 * row_spacing;
                for (i, idx) in nodes.iter().enumerate() {
                    if let Some(node) = self.graph.node_mut(*idx) {
                        let x = -width / 2.0 + i as f32 * col_spacing;
                        node.set_location(Pos2::new(x, y));
                    }
                }
            }
        }

        fn draw_hover_popup(&mut self, ui: &mut egui::Ui, node_idx: NodeIndex<u32>) {
            if let Some(class_info) = self.class_details.get(&node_idx).cloned() {
                // Get or create default size for this popup
                let default_size = Vec2::new(280.0, 350.0);
                let current_size = self.popup_sizes.get(&node_idx).copied().unwrap_or(default_size);
                
                let window_id = egui::Id::new(format!("node_popup_{}", node_idx.index()));
                
                let mut updated_description: Option<String> = None;
                let description_to_render = class_info.description.clone();
                let cache = &mut self.markdown_cache;
                
                let window_response = egui::Window::new(format!("📦 {}", &class_info.name))
                    .id(window_id)
                    .default_size(current_size)
                    .min_width(200.0)
                    .min_height(150.0)
                    .resizable(true)
                    .collapsible(true)
                    .show(ui.ctx(), |ui| {
                        // Use the full available rect for scrolling
                        let available_height = ui.available_height().max(100.0);
                        
                        egui::ScrollArea::vertical()
                            .max_height(available_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                            // Fields section
                            ui.collapsing("📋 Fields", |ui| {
                                for field in &class_info.fields {
                                    ui.horizontal(|ui| {
                                        ui.label("•");
                                        ui.monospace(field);
                                    });
                                }
                                if class_info.fields.is_empty() {
                                    ui.weak("No fields defined");
                                }
                            });
                            
                            ui.add_space(4.0);
                            
                            // Methods section
                            ui.collapsing("⚡ Methods", |ui| {
                                for method in &class_info.methods {
                                    ui.horizontal(|ui| {
                                        ui.label("•");
                                        ui.monospace(method);
                                    });
                                }
                                if class_info.methods.is_empty() {
                                    ui.weak("No methods defined");
                                }
                            });
                            
                            ui.add_space(4.0);
                            ui.separator();
                            
                            // Editable description with markdown
                            ui.collapsing("📝 Description", |ui| {
                                let mut description = description_to_render.clone();
                                let text_edit = egui::TextEdit::multiline(&mut description)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(4)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text("Add notes here (supports **bold**, *italic*, `code`)...");
                                let response = ui.add(text_edit);
                                
                                if response.changed() {
                                    updated_description = Some(description.clone());
                                }
                                
                                // Show markdown preview if there's content
                                if !description.is_empty() {
                                    ui.add_space(4.0);
                                    ui.separator();
                                    ui.label("Preview:");
                                    CommonMarkViewer::new().show(ui, cache, &description);
                                }
                            });
                        });
                    });
                
                // Store the new size from the window response
                if let Some(inner_response) = window_response {
                    let new_size = inner_response.response.rect.size();
                    self.popup_sizes.insert(node_idx, new_size);
                }
                
                // Update description if changed
                if let Some(new_desc) = updated_description {
                    if let Some(info) = self.class_details.get_mut(&node_idx) {
                        info.description = new_desc;
                    }
                }
            }
        }

        fn save_config(&self, ctx: &egui::Context) {
            ctx.data_mut(|data| {
                data.insert_persisted(egui::Id::new("app_config"), self.config.clone());
                data.insert_persisted(egui::Id::new("auto_save_config"), self.auto_save_config);
            });
        }

        fn draw_config_window(&mut self, ctx: &egui::Context) {
            let show_config = self.show_config_window;
            let mut show_window = show_config;
            egui::Window::new("⚙ Configuration")
                .open(&mut show_window)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    let mut config = self.config.clone();
                    let mut auto_save = self.auto_save_config;
                    let mut search = self.config_search.clone();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Search bar
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.text_edit_singleline(&mut search);
                            if ui.button("✖").clicked() {
                                search.clear();
                            }
                        });
                        ui.separator();

                        let search_lower = search.to_lowercase();
                        let matches = |text: &str| search_lower.is_empty() || text.to_lowercase().contains(&search_lower);

                        // Auto-save checkbox
                        ui.checkbox(&mut auto_save, "💾 Auto-save configuration");
                        
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save Now").clicked() {
                                self.config = config.clone();
                                self.auto_save_config = auto_save;
                                self.save_config(ctx);
                            }
                            if ui.button("↺ Reset to Defaults").clicked() {
                                config = AppConfig::default();
                                if auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            }
                        });
                        
                        ui.separator();

                        // Interaction Settings
                        if matches("interaction") || matches("dragging") || matches("hover") || matches("clicking") || matches("selection") {
                            ui.collapsing("🖱️ Interaction Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("dragging") {
                                    changed |= ui.checkbox(&mut config.dragging_enabled, "Enable Node Dragging").changed();
                                }
                                if matches("hover") {
                                    changed |= ui.checkbox(&mut config.hover_enabled, "Enable Hover Detection").changed();
                                }
                                if matches("clicking") || matches("node") {
                                    changed |= ui.checkbox(&mut config.node_clicking_enabled, "Enable Node Clicking").changed();
                                }
                                if matches("selection") || matches("node") {
                                    changed |= ui.checkbox(&mut config.node_selection_enabled, "Enable Node Selection").changed();
                                }
                                if matches("multi") || matches("selection") || matches("node") {
                                    changed |= ui.checkbox(&mut config.node_selection_multi_enabled, "Enable Multi-Node Selection").changed();
                                }
                                if matches("clicking") || matches("edge") {
                                    changed |= ui.checkbox(&mut config.edge_clicking_enabled, "Enable Edge Clicking").changed();
                                }
                                if matches("selection") || matches("edge") {
                                    changed |= ui.checkbox(&mut config.edge_selection_enabled, "Enable Edge Selection").changed();
                                }
                                if matches("multi") || matches("selection") || matches("edge") {
                                    changed |= ui.checkbox(&mut config.edge_selection_multi_enabled, "Enable Multi-Edge Selection").changed();
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // Simulation/Layout Settings
                        if matches("simulation") || matches("layout") || matches("force") || matches("animate") {
                            ui.collapsing("⚡ Simulation/Layout Settings", |ui| {
                                ui.label("💡 Force-Directed Layout:");
                                ui.label("  The graph uses Fruchterman-Reingold algorithm");
                                ui.label("  for automatic node positioning.");
                                ui.separator();
                                
                                ui.label("⚙️ Available Parameters:");
                                ui.label("  • dt: Time step (default: 0.05)");
                                ui.label("  • damping: Velocity damping (default: 0.3)");
                                ui.label("  • max_step: Max displacement per step (default: 10.0)");
                                ui.label("  • k_scale: Spring constant scale (default: 1.0)");
                                ui.label("  • c_attract: Attraction force (default: 1.0)");
                                ui.label("  • c_repulse: Repulsion force (default: 1.0)");
                                ui.separator();
                                
                                ui.label("📋 Layout Types:");
                                ui.label("  • Force-Directed (current)");
                                ui.label("  • Random");
                                ui.label("  • Hierarchical");
                                ui.separator();
                                
                                ui.label("❗ Note: Interactive simulation controls");
                                ui.label("require access to the layout state.");
                                ui.label("Advanced controls coming in future update!");
                            });
                        }

                        // Graph Generation Settings
                        if matches("graph") || matches("nodes") || matches("edges") || matches("vertices") || matches("generate") {
                            ui.collapsing("🔨 Graph Generation", |ui| {
                                let mut changed = false;
                                let old_nodes = config.num_nodes;
                                let old_edges = config.num_edges;
                                
                                ui.horizontal(|ui| {
                                    ui.label("Number of Nodes:");
                                    changed |= ui.add(egui::Slider::new(&mut config.num_nodes, 1..=200).suffix(" nodes")).changed();
                                    changed |= ui.add(egui::DragValue::new(&mut config.num_nodes).range(1..=1000)).changed();
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Number of Edges:");
                                    changed |= ui.add(egui::Slider::new(&mut config.num_edges, 0..=400).suffix(" edges")).changed();
                                    changed |= ui.add(egui::DragValue::new(&mut config.num_edges).range(0..=2000)).changed();
                                });
                                
                                if changed {
                                    self.config = config.clone();
                                    if auto_save {
                                        self.auto_save_config = auto_save;
                                        self.save_config(ctx);
                                    }
                                    
                                    // Regenerate graph if values changed
                                    if old_nodes != config.num_nodes || old_edges != config.num_edges {
                                        self.regenerate_graph(config.num_nodes, config.num_edges);
                                    }
                                }
                                
                                ui.separator();
                                if ui.button("🔄 Regenerate Graph").clicked() {
                                    self.regenerate_graph(config.num_nodes, config.num_edges);
                                }
                                ui.label("💡 Adjust sliders or click to regenerate with new structure");
                            });
                        }

                        // Navigation Settings
                        if matches("navigation") || matches("zoom") || matches("pan") || matches("fit") {
                            ui.collapsing("🧭 Navigation Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("fit") || matches("screen") {
                                    changed |= ui.checkbox(&mut config.fit_to_screen_enabled, "Fit to Screen").changed();
                                }
                                if matches("zoom") || matches("pan") {
                                    changed |= ui.checkbox(&mut config.zoom_and_pan_enabled, "Enable Zoom & Pan").changed();
                                }
                                if matches("padding") || matches("fit") {
                                    ui.horizontal(|ui| {
                                        ui.label("Fit to Screen Padding:");
                                        changed |= ui.add(egui::Slider::new(&mut config.fit_to_screen_padding, 0.0..=0.5).suffix("x")).changed();
                                        changed |= ui.add(egui::DragValue::new(&mut config.fit_to_screen_padding).range(0.0..=0.5).speed(0.01)).changed();
                                    });
                                }
                                if matches("zoom") || matches("speed") {
                                    ui.horizontal(|ui| {
                                        ui.label("Zoom Speed:");
                                        changed |= ui.add(egui::Slider::new(&mut config.zoom_speed, 0.01..=1.0).logarithmic(true)).changed();
                                        changed |= ui.add(egui::DragValue::new(&mut config.zoom_speed).range(0.01..=1.0).speed(0.01)).changed();
                                    });
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // Style Settings
                        if matches("style") || matches("label") || matches("appearance") {
                            ui.collapsing("🎨 Style Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("label") {
                                    changed |= ui.checkbox(&mut config.labels_always, "Always Show Labels").changed();
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // Grid and Axes Settings
                        if matches("grid") || matches("axis") || matches("axes") || matches("origin") || matches("reference") {
                            ui.collapsing("📐 Grid & Axes", |ui| {
                                let mut changed = false;
                                
                                if matches("grid") {
                                    changed |= ui.checkbox(&mut config.show_grid, "Show Grid").changed();
                                    if config.show_grid {
                                        ui.horizontal(|ui| {
                                            ui.label("Grid Spacing:");
                                            changed |= ui.add(egui::Slider::new(&mut config.grid_spacing, 20.0..=200.0)).changed();
                                        });
                                    }
                                }
                                
                                if matches("axis") || matches("axes") || matches("origin") {
                                    changed |= ui.checkbox(&mut config.show_axes, "Show Axes & Origin").changed();
                                    ui.label("  🔴 X axis (red)   🟢 Y axis (green)");
                                    if config.visualization_mode == VisualizationMode::ThreeD {
                                        ui.label("  🔵 Z axis (blue) - 3D mode");
                                    }
                                    ui.label("  🟡 Origin point (0,0)");
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // Color Settings
                        if matches("color") || matches("theme") || matches("appearance") {
                            ui.collapsing("🎨 Color Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("node") || matches("color") {
                                    ui.label("Node Color:");
                                    let mut color = Color32::from_rgb(config.node_color[0], config.node_color[1], config.node_color[2]);
                                    if ui.color_edit_button_srgba(&mut color).changed() {
                                        config.node_color = [color.r(), color.g(), color.b()];
                                        changed = true;
                                    }
                                    
                                    ui.label("Node Hover Color:");
                                    let mut hover_color = Color32::from_rgb(config.node_hover_color[0], config.node_hover_color[1], config.node_hover_color[2]);
                                    if ui.color_edit_button_srgba(&mut hover_color).changed() {
                                        config.node_hover_color = [hover_color.r(), hover_color.g(), hover_color.b()];
                                        changed = true;
                                    }
                                    
                                    ui.label("Node Selected Color:");
                                    let mut selected_color = Color32::from_rgb(config.node_selected_color[0], config.node_selected_color[1], config.node_selected_color[2]);
                                    if ui.color_edit_button_srgba(&mut selected_color).changed() {
                                        config.node_selected_color = [selected_color.r(), selected_color.g(), selected_color.b()];
                                        changed = true;
                                    }
                                }
                                
                                if matches("edge") || matches("color") {
                                    ui.label("Edge Color:");
                                    let mut edge_color = Color32::from_rgb(config.edge_color[0], config.edge_color[1], config.edge_color[2]);
                                    if ui.color_edit_button_srgba(&mut edge_color).changed() {
                                        config.edge_color = [edge_color.r(), edge_color.g(), edge_color.b()];
                                        changed = true;
                                    }
                                    
                                    ui.label("Edge Selected Color:");
                                    let mut edge_selected = Color32::from_rgb(config.edge_selected_color[0], config.edge_selected_color[1], config.edge_selected_color[2]);
                                    if ui.color_edit_button_srgba(&mut edge_selected).changed() {
                                        config.edge_selected_color = [edge_selected.r(), edge_selected.g(), edge_selected.b()];
                                        changed = true;
                                    }
                                }
                                
                                if matches("background") || matches("color") {
                                    ui.label("Background Color:");
                                    let mut bg_color = Color32::from_rgb(config.background_color[0], config.background_color[1], config.background_color[2]);
                                    if ui.color_edit_button_srgba(&mut bg_color).changed() {
                                        config.background_color = [bg_color.r(), bg_color.g(), bg_color.b()];
                                        changed = true;
                                    }
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // UI Settings
                        if matches("ui") || matches("window") || matches("popup") || matches("panel") {
                            ui.collapsing("🖼️ UI Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("popup") || matches("window") || matches("hover") {
                                    ui.label("Hover Popup Size:");
                                    let mut size_percent = (config.hover_window_size * 100.0) as i32;
                                    if ui.add(egui::Slider::new(&mut size_percent, 5..=50).suffix("%")).changed() || 
                                       ui.add(egui::DragValue::new(&mut size_percent).range(5..=50).suffix("%")).changed() {
                                        config.hover_window_size = size_percent as f32 / 100.0;
                                        changed = true;
                                    }
                                }
                                
                                if matches("panel") || matches("side") {
                                    changed |= ui.checkbox(&mut config.show_side_panel, "Show Side Panel").changed();
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }
                    });
                    
                    // Update self after window closes
                    self.config = config;
                    self.auto_save_config = auto_save;
                    self.config_search = search;
                });
            self.show_config_window = show_window;
        }

        fn draw_stress_test_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
            let current_time = ctx.input(|i| i.time);
            
            ui.heading("🚨 DDoS Stress Test Simulator");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Attack Pattern:");
                egui::ComboBox::from_id_salt("attack_pattern")
                    .selected_text(self.attack_pattern.name())
                    .show_ui(ui, |ui| {
                        for pattern in AttackPattern::all() {
                            ui.selectable_value(&mut self.attack_pattern, pattern, pattern.name());
                        }
                    });
            });
            
            ui.label(format!("Description: {}", self.attack_pattern.description()));
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Intensity:");
                ui.add(egui::Slider::new(&mut self.attack_intensity, 1.0..=100.0).suffix("%"));
                ui.add(egui::DragValue::new(&mut self.attack_intensity).range(1.0..=100.0).suffix("%"));
            });
            
            ui.horizontal(|ui| {
                ui.label("Max Requests/sec:");
                ui.add(egui::Slider::new(&mut self.max_requests_per_sec, 100..=10000).suffix(" req/s"));
                ui.add(egui::DragValue::new(&mut self.max_requests_per_sec).range(100..=50000));
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                if self.stress_active {
                    if ui.button("⏸ Stop Attack").clicked() {
                        self.stop_stress_test(current_time);
                    }
                } else {
                    if ui.button("▶ Start Attack").clicked() {
                        self.stress_active = true;
                        self.stress_metrics.start_time = None; // Will be set on first update
                        self.add_log(LogLevel::Info, format!("🎯 Preparing {} attack...", self.attack_pattern.name()), current_time);
                    }
                }
                
                if ui.button("🔄 Reset Metrics").clicked() {
                    self.reset_stress_metrics(current_time);
                }
            });
            
            ui.separator();
            ui.heading("📊 Real-Time Statistics");
            ui.separator();
            
            egui::Grid::new("metrics_grid")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("⏱️ Elapsed Time:");
                    ui.label(format!("{:.1} seconds", self.stress_metrics.elapsed_time));
                    ui.end_row();
                    
                    ui.label("📨 Total Requests:");
                    ui.label(format!("{}", self.stress_metrics.total_requests));
                    ui.end_row();
                    
                    ui.label("✅ Successful:");
                    ui.colored_label(Color32::from_rgb(100, 255, 100), 
                        format!("{} ({:.1}%)", 
                            self.stress_metrics.successful_requests,
                            if self.stress_metrics.total_requests > 0 {
                                (self.stress_metrics.successful_requests as f64 / self.stress_metrics.total_requests as f64) * 100.0
                            } else { 0.0 }
                        )
                    );
                    ui.end_row();
                    
                    ui.label("❌ Failed:");
                    ui.colored_label(Color32::from_rgb(255, 100, 100), 
                        format!("{} ({:.1}%)", 
                            self.stress_metrics.failed_requests,
                            if self.stress_metrics.total_requests > 0 {
                                (self.stress_metrics.failed_requests as f64 / self.stress_metrics.total_requests as f64) * 100.0
                            } else { 0.0 }
                        )
                    );
                    ui.end_row();
                    
                    ui.label("📖 Read Operations:");
                    ui.label(format!("{}", self.stress_metrics.read_operations));
                    ui.end_row();
                    
                    ui.label("✏️ Write Operations:");
                    ui.label(format!("{}", self.stress_metrics.write_operations));
                    ui.end_row();
                    
                    ui.label("📤 Bytes Sent:");
                    ui.label(format!("{:.2} MB", self.stress_metrics.bytes_sent as f64 / (1024.0 * 1024.0)));
                    ui.end_row();
                    
                    ui.label("📥 Bytes Received:");
                    ui.label(format!("{:.2} MB", self.stress_metrics.bytes_received as f64 / (1024.0 * 1024.0)));
                    ui.end_row();
                    
                    ui.label("📊 Current Throughput:");
                    ui.colored_label(
                        if self.stress_metrics.current_throughput > 100.0 { Color32::from_rgb(255, 100, 100) }
                        else if self.stress_metrics.current_throughput > 50.0 { Color32::from_rgb(255, 200, 100) }
                        else { Color32::from_rgb(100, 255, 100) },
                        format!("{:.2} MB/s", self.stress_metrics.current_throughput)
                    );
                    ui.end_row();
                    
                    ui.label("🔥 Peak Throughput:");
                    ui.label(format!("{:.2} MB/s", self.stress_metrics.peak_throughput));
                    ui.end_row();
                    
                    ui.label("⏰ Avg Response Time:");
                    ui.label(format!("{:.1} ms", self.stress_metrics.avg_response_time));
                    ui.end_row();
                });
            
            ui.separator();
            ui.heading("📈 Throughput Graph");
            ui.separator();
            
            self.draw_throughput_graph(ui);
        }

        fn draw_throughput_graph(&self, ui: &mut egui::Ui) {
            let desired_size = Vec2::new(ui.available_width(), 200.0);
            let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
            
            let rect = response.rect;
            
            // Draw background
            painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 25));
            
            // Draw border
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(60, 60, 70)), egui::StrokeKind::Outside);
            
            if self.throughput_history.len() < 2 {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Waiting for data...",
                    FontId::proportional(14.0),
                    Color32::from_rgb(150, 150, 150),
                );
                return;
            }
            
            // Find min/max for scaling
            let max_throughput = self.throughput_history
                .iter()
                .map(|p| p.value)
                .fold(0.0f64, |a, b| a.max(b))
                .max(1.0);
            
            let max_time = self.throughput_history
                .back()
                .map(|p| p.time)
                .unwrap_or(1.0);
            
            // Draw grid lines
            for i in 0..5 {
                let y = rect.min.y + (rect.height() * i as f32 / 4.0);
                painter.line_segment(
                    [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                    Stroke::new(0.5, Color32::from_rgb(40, 40, 50)),
                );
                
                let value = max_throughput * (4.0 - i as f64) / 4.0;
                painter.text(
                    Pos2::new(rect.min.x + 5.0, y),
                    egui::Align2::LEFT_CENTER,
                    format!("{:.1}", value),
                    FontId::proportional(10.0),
                    Color32::from_rgb(150, 150, 150),
                );
            }
            
            // Draw line graph
            let points: Vec<Pos2> = self.throughput_history
                .iter()
                .map(|p| {
                    let x = rect.min.x + (p.time / max_time) as f32 * rect.width();
                    let y = rect.max.y - (p.value / max_throughput) as f32 * rect.height();
                    Pos2::new(x, y)
                })
                .collect();
            
            // Draw filled area under the line
            if points.len() >= 2 {
                let mut fill_points = points.clone();
                fill_points.push(Pos2::new(points.last().unwrap().x, rect.max.y));
                fill_points.push(Pos2::new(points[0].x, rect.max.y));
                
                painter.add(Shape::convex_polygon(
                    fill_points,
                    Color32::from_rgba_unmultiplied(100, 200, 255, 30),
                    Stroke::NONE,
                ));
            }
            
            // Draw the line
            for i in 0..points.len().saturating_sub(1) {
                painter.line_segment(
                    [points[i], points[i + 1]],
                    Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                );
            }
            
            // Draw labels
            painter.text(
                Pos2::new(rect.center().x, rect.max.y + 10.0),
                egui::Align2::CENTER_TOP,
                format!("Time (seconds) - Max: {:.1} MB/s", max_throughput),
                FontId::proportional(12.0),
                Color32::from_rgb(200, 200, 200),
            );
        }

        fn draw_neural_network_tab(&mut self, ui: &mut egui::Ui) {
            ui.heading("🧠 Neural Network Simulator");
            ui.separator();
            
            ui.label("Configure the neural network architecture:");
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Input Layers:");
                ui.add(egui::Slider::new(&mut self.config.nn_config.input_layers, 1..=3));
            });
            
            ui.horizontal(|ui| {
                ui.label("Neurons per Input Layer:");
                ui.add(egui::Slider::new(&mut self.config.nn_config.neurons_per_input_layer, 1..=10));
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Hidden Layers:");
                ui.add(egui::Slider::new(&mut self.config.nn_config.hidden_layers, 1..=5));
            });
            
            ui.horizontal(|ui| {
                ui.label("Neurons per Hidden Layer:");
                ui.add(egui::Slider::new(&mut self.config.nn_config.neurons_per_hidden_layer, 2..=12));
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Output Layers:");
                ui.add(egui::Slider::new(&mut self.config.nn_config.output_layers, 1..=3));
            });
            
            ui.horizontal(|ui| {
                ui.label("Neurons per Output Layer:");
                ui.add(egui::Slider::new(&mut self.config.nn_config.neurons_per_output_layer, 1..=10));
            });
            
            ui.separator();
            ui.heading("Simulation Settings");
            
            ui.horizontal(|ui| {
                ui.label("Fire Rate (Hz):");
                ui.add(egui::Slider::new(&mut self.config.nn_config.fire_rate, 0.1..=10.0));
            });
            
            ui.horizontal(|ui| {
                ui.label("Propagation Speed:");
                ui.add(egui::Slider::new(&mut self.config.nn_config.fire_propagation_speed, 0.1..=2.0));
            });
            
            ui.separator();
            ui.heading("Color Settings");
            
            ui.horizontal(|ui| {
                ui.label("Inactive Neuron:");
                let mut color = Color32::from_rgb(
                    self.config.nn_config.neuron_inactive_color[0],
                    self.config.nn_config.neuron_inactive_color[1],
                    self.config.nn_config.neuron_inactive_color[2],
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    self.config.nn_config.neuron_inactive_color = [color.r(), color.g(), color.b()];
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Firing Neuron:");
                let mut color = Color32::from_rgb(
                    self.config.nn_config.neuron_firing_color[0],
                    self.config.nn_config.neuron_firing_color[1],
                    self.config.nn_config.neuron_firing_color[2],
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    self.config.nn_config.neuron_firing_color = [color.r(), color.g(), color.b()];
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Input Neuron:");
                let mut color = Color32::from_rgb(
                    self.config.nn_config.input_neuron_color[0],
                    self.config.nn_config.input_neuron_color[1],
                    self.config.nn_config.input_neuron_color[2],
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    self.config.nn_config.input_neuron_color = [color.r(), color.g(), color.b()];
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Output Neuron:");
                let mut color = Color32::from_rgb(
                    self.config.nn_config.output_neuron_color[0],
                    self.config.nn_config.output_neuron_color[1],
                    self.config.nn_config.output_neuron_color[2],
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    self.config.nn_config.output_neuron_color = [color.r(), color.g(), color.b()];
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Synapse:");
                let mut color = Color32::from_rgb(
                    self.config.nn_config.synapse_color[0],
                    self.config.nn_config.synapse_color[1],
                    self.config.nn_config.synapse_color[2],
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    self.config.nn_config.synapse_color = [color.r(), color.g(), color.b()];
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Active Synapse:");
                let mut color = Color32::from_rgb(
                    self.config.nn_config.synapse_active_color[0],
                    self.config.nn_config.synapse_active_color[1],
                    self.config.nn_config.synapse_active_color[2],
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    self.config.nn_config.synapse_active_color = [color.r(), color.g(), color.b()];
                }
            });
            
            ui.separator();
            
            ui.checkbox(&mut self.config.nn_config.show_neuron_values, "Show Neuron Values");
            
            ui.separator();
            ui.add_space(10.0);
            
            if ui.button("🔄 Generate Neural Network").clicked() {
                self.generate_neural_network();
                self.config.fit_to_screen_enabled = true;
            }
            
            ui.add_space(10.0);
            
            if self.neural_network.is_some() {
                ui.label(format!(
                    "Network: {} input, {} hidden, {} output layers",
                    self.config.nn_config.input_layers,
                    self.config.nn_config.hidden_layers,
                    self.config.nn_config.output_layers
                ));
                
                let total_neurons: usize = 
                    self.config.nn_config.input_layers * self.config.nn_config.neurons_per_input_layer +
                    self.config.nn_config.hidden_layers * self.config.nn_config.neurons_per_hidden_layer +
                    self.config.nn_config.output_layers * self.config.nn_config.neurons_per_output_layer;
                
                ui.label(format!("Total Neurons: {}", total_neurons));
                
                let firing_count = self.neuron_states.values().filter(|s| s.is_firing).count();
                ui.label(format!("Firing: {}", firing_count));
            }
        }

        fn draw_logs_tab(&self, ui: &mut egui::Ui) {
            ui.heading("📋 System Logs");
            ui.separator();
            
            ui.label(format!("Total entries: {} (max: {})", self.logs.len(), self.max_log_entries));
            ui.separator();
            
            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for entry in &self.logs {
                        ui.horizontal(|ui| {
                            ui.colored_label(entry.level.color(), entry.level.prefix());
                            ui.label(format!("[{:.2}s]", entry.timestamp));
                            ui.label(&entry.message);
                        });
                    }
                });
        }
    }

    impl App for CodeAnalyzerApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Top menu bar
            egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    // Visualization mode dropdown
                    ui.label("View:");
                    egui::ComboBox::from_id_salt("viz_mode")
                        .selected_text(self.config.visualization_mode.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.visualization_mode, VisualizationMode::TwoD, "2D View");
                            ui.selectable_value(&mut self.config.visualization_mode, VisualizationMode::ThreeD, "3D View");
                        });
                    
                    ui.separator();
                    
                    ui.menu_button("📁 File", |ui| {
                        if ui.button("📂 Import Graph (JSON)").clicked() {
                            self.trigger_file_upload();
                            ui.close();
                        }
                        if ui.button("💾 Export Graph").clicked() {
                            self.show_export_dialog = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("💾 Save Configuration").clicked() {
                            self.save_config(ctx);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("🔄 Load Example Graph").clicked() {
                            self.regenerate_graph(6, 8);
                            ui.close();
                        }
                    });

                    ui.menu_button("⚙️ Configuration", |ui| {
                        if ui.button("Open Settings").clicked() {
                            self.show_config_window = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("↺ Reset to Defaults").clicked() {
                            self.config = AppConfig::default();
                            if self.auto_save_config {
                                self.save_config(ctx);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("🎨 Colors", |ui| {
                        if ui.button("Change Colors").clicked() {
                            self.show_color_picker = true;
                            ui.close();
                        }
                    });

                    ui.menu_button("ℹ️ Help", |ui| {
                        ui.label("Code Analyzer v1.0");
                        ui.separator();
                        ui.label("Legend:");
                        ui.label("  1:1 = One-to-One");
                        ui.label("  1:N = One-to-Many");
                        ui.label("  N:N = Many-to-Many");
                    });
                });
            });

            // Config window
            if self.show_config_window {
                self.draw_config_window(ctx);
            }

            // Color picker window
            if self.show_color_picker {
                let mut show_picker = self.show_color_picker;
                let mut config = self.config.clone();
                
                egui::Window::new("🎨 Color Customization")
                    .open(&mut show_picker)
                    .resizable(true)
                    .default_width(300.0)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.heading("Node Colors");
                            ui.horizontal(|ui| {
                                ui.label("Default:");
                                let mut color = Color32::from_rgb(config.node_color[0], config.node_color[1], config.node_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.node_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Hover:");
                                let mut color = Color32::from_rgb(config.node_hover_color[0], config.node_hover_color[1], config.node_hover_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.node_hover_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Selected:");
                                let mut color = Color32::from_rgb(config.node_selected_color[0], config.node_selected_color[1], config.node_selected_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.node_selected_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.separator();
                            ui.heading("Edge Colors");
                            
                            ui.horizontal(|ui| {
                                ui.label("Default:");
                                let mut color = Color32::from_rgb(config.edge_color[0], config.edge_color[1], config.edge_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.edge_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Selected:");
                                let mut color = Color32::from_rgb(config.edge_selected_color[0], config.edge_selected_color[1], config.edge_selected_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.edge_selected_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.separator();
                            ui.heading("Background");
                            
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                let mut color = Color32::from_rgb(config.background_color[0], config.background_color[1], config.background_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.background_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("💾 Save").clicked() {
                                    self.config = config.clone();
                                    self.save_config(ctx);
                                }
                                if ui.button("↺ Reset to Defaults").clicked() {
                                    let default = AppConfig::default();
                                    config.node_color = default.node_color;
                                    config.node_hover_color = default.node_hover_color;
                                    config.node_selected_color = default.node_selected_color;
                                    config.edge_color = default.edge_color;
                                    config.edge_selected_color = default.edge_selected_color;
                                    config.background_color = default.background_color;
                                }
                            });
                        });
                    });
                
                self.config = config;
                self.show_color_picker = show_picker;
            }

            // File upload dialog
            if self.show_file_dialog {
                let mut show_dialog = self.show_file_dialog;
                egui::Window::new("📂 Import Graph")
                    .open(&mut show_dialog)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(&self.file_dialog_message);
                        ui.separator();
                        
                        ui.label("Expected JSON format:");
                        ui.code(r#"{
  "nodes": [
    {
      "name": "ClassName",
      "methods": ["method1()", "method2()"],
      "fields": ["field1: type", "field2: type"]
    }
  ],
  "edges": [
    {
      "from": 0,
      "to": 1,
      "relationship": "1:N"
    }
  ]
}"#);
                        
                        ui.separator();
                        
                        if ui.button("📄 Paste JSON").clicked() {
                            // Clipboard paste not available in WASM
                            self.file_dialog_message = "Clipboard paste not available in web. Use file upload instead.".to_string();
                        }
                        
                        ui.horizontal(|ui| {
                            if ui.button("❌ Cancel").clicked() {
                                self.show_file_dialog = false;
                            }
                        });
                    });
                self.show_file_dialog = show_dialog;
            }

            // Export dialog
            if self.show_export_dialog {
                let mut show_dialog = self.show_export_dialog;
                egui::Window::new("💾 Export Graph")
                    .open(&mut show_dialog)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("Select export format:");
                        ui.separator();
                        
                        ui.radio_value(&mut self.export_format, ExportFormat::Json, "📄 JSON - JavaScript Object Notation");
                        ui.radio_value(&mut self.export_format, ExportFormat::Csv, "📊 CSV - Comma Separated Values");
                        ui.radio_value(&mut self.export_format, ExportFormat::Graphviz, "🔷 Graphviz DOT - Graph Description Language");
                        
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            if ui.button("💾 Export").clicked() {
                                let content = match self.export_format {
                                    ExportFormat::Json => self.export_graph_json(),
                                    ExportFormat::Csv => self.export_graph_csv(),
                                    ExportFormat::Graphviz => self.export_graph_graphviz(),
                                };
                                
                                let filename = format!("graph_export.{}", self.export_format.extension());
                                self.download_file(&filename, &content);
                                self.show_export_dialog = false;
                            }
                            
                            if ui.button("❌ Cancel").clicked() {
                                self.show_export_dialog = false;
                            }
                        });
                    });
                self.show_export_dialog = show_dialog;
            }

            // 3D Controls panel (only show in 3D mode)
            if self.config.visualization_mode == VisualizationMode::ThreeD {
                egui::Window::new("🎮 3D Visualization Settings")
                    .default_pos([10.0, 100.0])
                    .default_width(300.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Rotation Settings
                            ui.collapsing("🔄 Rotation", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("X Axis:");
                                    if ui.add(egui::Slider::new(&mut self.config.rotation_x, -180.0..=180.0).suffix("°")).changed() ||
                                       ui.add(egui::DragValue::new(&mut self.config.rotation_x).range(-180.0..=180.0).suffix("°").speed(1.0)).changed() {
                                        ctx.request_repaint();
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Y Axis:");
                                    if ui.add(egui::Slider::new(&mut self.config.rotation_y, -180.0..=180.0).suffix("°")).changed() ||
                                       ui.add(egui::DragValue::new(&mut self.config.rotation_y).range(-180.0..=180.0).suffix("°").speed(1.0)).changed() {
                                        ctx.request_repaint();
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Z Axis:");
                                    if ui.add(egui::Slider::new(&mut self.config.rotation_z, -180.0..=180.0).suffix("°")).changed() ||
                                       ui.add(egui::DragValue::new(&mut self.config.rotation_z).range(-180.0..=180.0).suffix("°").speed(1.0)).changed() {
                                        ctx.request_repaint();
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                
                                ui.separator();
                                
                                // Prominent stop button if rotation is active
                                if self.config.auto_rotate {
                                    if ui.button("⏹ Stop Rotation").clicked() {
                                        self.config.auto_rotate = false;
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                }
                                
                                if ui.checkbox(&mut self.config.auto_rotate, "Auto Rotate").changed() {
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if self.config.auto_rotate {
                                    ui.horizontal(|ui| {
                                        ui.label("Speed:");
                                        if ui.add(egui::Slider::new(&mut self.config.rotation_speed, 0.1..=5.0).logarithmic(true)).changed() ||
                                           ui.add(egui::DragValue::new(&mut self.config.rotation_speed).range(0.1..=5.0).speed(0.1)).changed() {
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    });
                                }
                            });
                            
                            // Positioning Settings
                            ui.collapsing("📐 Positioning", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Layer Spacing:");
                                    if ui.add(egui::Slider::new(&mut self.config.z_spacing, 10.0..=500.0).suffix(" units")).changed() ||
                                       ui.add(egui::DragValue::new(&mut self.config.z_spacing).range(10.0..=500.0).suffix(" units").speed(5.0)).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Layer Offset:");
                                    if ui.add(egui::Slider::new(&mut self.config.layer_offset, -200.0..=200.0).suffix(" units")).changed() ||
                                       ui.add(egui::DragValue::new(&mut self.config.layer_offset).range(-200.0..=200.0).suffix(" units").speed(5.0)).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Perspective:");
                                    if ui.add(egui::Slider::new(&mut self.config.perspective_strength, 0.0..=0.01).logarithmic(true)).changed() ||
                                       ui.add(egui::DragValue::new(&mut self.config.perspective_strength).range(0.0..=0.01).speed(0.0001)).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                            });
                            
                            // Simulation Control
                            ui.collapsing("⚙️ Simulation Control", |ui| {
                                ui.label("Control the force-directed layout simulation:");
                                ui.separator();
                                
                                ui.horizontal(|ui| {
                                    if self.config.simulation_running {
                                        if ui.button("⏸ Pause Simulation").clicked() {
                                            self.config.simulation_running = false;
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    } else {
                                        if ui.button("▶ Resume Simulation").clicked() {
                                            self.config.simulation_running = true;
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    }
                                });
                                
                                ui.label(if self.config.simulation_running {
                                    "✓ Simulation is running - nodes will move to optimal positions"
                                } else {
                                    "⏸ Simulation is paused - nodes will stay in place"
                                });
                                
                                ui.separator();
                                ui.label("💡 Tip: Pause the simulation to manually arrange nodes,");
                                ui.label("   then resume to let them settle into stable positions.");
                            });
                            
                            // Visual Effects Settings
                            ui.collapsing("✨ Visual Effects", |ui| {
                                if ui.checkbox(&mut self.config.depth_fade_enabled, "Depth Fade").changed() {
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if self.config.depth_fade_enabled {
                                    ui.horizontal(|ui| {
                                        ui.label("    Strength:");
                                        if ui.add(egui::Slider::new(&mut self.config.depth_fade_strength, 0.0..=0.005).logarithmic(true)).changed() ||
                                           ui.add(egui::DragValue::new(&mut self.config.depth_fade_strength).range(0.0..=0.005).speed(0.0001)).changed() {
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    });
                                }
                                
                                if ui.checkbox(&mut self.config.depth_scale_enabled, "Depth Scaling").changed() {
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if self.config.depth_scale_enabled {
                                    ui.horizontal(|ui| {
                                        ui.label("    Strength:");
                                        if ui.add(egui::Slider::new(&mut self.config.depth_scale_strength, 0.0..=0.01).logarithmic(true)).changed() ||
                                           ui.add(egui::DragValue::new(&mut self.config.depth_scale_strength).range(0.0..=0.01).speed(0.0001)).changed() {
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    });
                                }
                                
                                ui.separator();
                                if ui.checkbox(&mut self.config.use_sphere_rendering, "Render as Spheres (3D)").changed() {
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if self.config.use_sphere_rendering {
                                    ui.label("    Nodes are rendered as shaded 3D spheres");
                                } else {
                                    ui.label("    Nodes are rendered as flat circles");
                                }
                            });
                            
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("↺ Reset All").clicked() {
                                    let defaults = AppConfig::default();
                                    self.config.rotation_x = defaults.rotation_x;
                                    self.config.rotation_y = defaults.rotation_y;
                                    self.config.rotation_z = defaults.rotation_z;
                                    self.config.auto_rotate = defaults.auto_rotate;
                                    self.config.rotation_speed = defaults.rotation_speed;
                                    self.config.z_spacing = defaults.z_spacing;
                                    self.config.layer_offset = defaults.layer_offset;
                                    self.config.perspective_strength = defaults.perspective_strength;
                                    self.config.depth_fade_enabled = defaults.depth_fade_enabled;
                                    self.config.depth_fade_strength = defaults.depth_fade_strength;
                                    self.config.depth_scale_enabled = defaults.depth_scale_enabled;
                                    self.config.depth_scale_strength = defaults.depth_scale_strength;
                                    self.config.use_sphere_rendering = defaults.use_sphere_rendering;
                                    self.config.simulation_running = defaults.simulation_running;
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if ui.button("💾 Save").clicked() {
                                    self.save_config(ctx);
                                }
                            });
                        });
                    });
            }

            // Side panel with tabs
            if self.config.show_side_panel {
                egui::SidePanel::right("side_panel").min_width(250.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.current_tab, AppTab::Graph, "📊 Graph");
                        ui.selectable_value(&mut self.current_tab, AppTab::StressTest, "🚨 Stress Test");
                        ui.selectable_value(&mut self.current_tab, AppTab::Logs, "📋 Logs");
                        ui.selectable_value(&mut self.current_tab, AppTab::NeuralNetwork, "🧠 Neural Net");
                    });
                    ui.separator();

                    match self.current_tab {
                        AppTab::Graph => {
                            ui.heading("📊 Classes");
                            ui.separator();

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (idx, info) in &self.class_details {
                                    if ui.button(&info.name).clicked() {
                                        if let Some(node) = self.graph.node_mut(*idx) {
                                            node.set_selected(true);
                                        }
                                    }
                                }
                            });
                        },
                        AppTab::StressTest => {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.draw_stress_test_tab(ctx, ui);
                            });
                        },
                        AppTab::Logs => {
                            self.draw_logs_tab(ui);
                        },
                        AppTab::NeuralNetwork => {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.draw_neural_network_tab(ui);
                            });
                        },
                    }
                    
                    ui.separator();
                    ui.add_space(10.0);
                    
                    if ui.button("🎯 Center Graph").clicked() {
                        self.config.fit_to_screen_enabled = true;
                        self.fit_to_screen_counter = 3; // Keep enabled for 3 frames
                        ctx.request_repaint();
                    }
                    
                    ui.separator();
                    ui.label("📐 View Options:");
                    ui.checkbox(&mut self.config.show_grid, "Show Grid");
                    ui.checkbox(&mut self.config.show_axes, "Show Axes & Origin");
                    let is_2d = self.config.visualization_mode == VisualizationMode::TwoD;
                    let hier_response = ui.add_enabled(is_2d, egui::Button::new("↳ Arrange Hierarchically"));
                    if !is_2d {
                        hier_response.clone().on_hover_text("Available only in 2D view");
                    }
                    if hier_response.clicked()
                    {
                        self.apply_hierarchical_layout();
                        self.config.fit_to_screen_enabled = true;
                        self.fit_to_screen_counter = 3;
                        ctx.request_repaint();
                    }
                    
                    ui.separator();
                    
                    if ui.button("⚙️ Settings").clicked() {
                        self.show_config_window = true;
                    }
                });
            }

            // Run stress test simulation
            if self.stress_active {
                self.simulate_stress_test(ctx);
            }
            
            // Run neural network simulation
            if self.current_tab == AppTab::NeuralNetwork && self.neural_network.is_some() {
                self.simulate_neural_network(ctx);
            }

            // Auto-rotation in 3D mode (only when explicitly enabled)
            if self.config.visualization_mode == VisualizationMode::ThreeD {
                if self.config.auto_rotate {
                    self.config.rotation_y += self.config.rotation_speed * 0.5;
                    if self.config.rotation_y > 180.0 {
                        self.config.rotation_y -= 360.0;
                    }
                    ctx.request_repaint();
                }
            }

            // Update node and edge colors from config, and apply 3D positioning
            let node_indices: Vec<_> = self.graph.g().node_indices().collect();
            let pivot = if node_indices.is_empty() {
                self.graph_pivot
            } else {
                let mut sum = egui::Vec2::ZERO;
                let mut count = 0.0f32;
                for idx in &node_indices {
                    if let Some(node) = self.graph.node(*idx) {
                        sum += node.location().to_vec2();
                        count += 1.0;
                    }
                }
                if count > 0.0 {
                    (sum / count).to_pos2()
                } else {
                    self.graph_pivot
                }
            };
            self.graph_pivot = pivot;
            for (layer_idx, idx) in node_indices.iter().enumerate() {
                if let Some(node) = self.graph.node_mut(*idx) {
                    node.display_mut().node_color = self.config.node_color;
                    node.display_mut().hover_color = self.config.node_hover_color;
                    node.display_mut().selected_color = self.config.node_selected_color;
                    node.display_mut().use_sphere_rendering = self.config.use_sphere_rendering;
                    
                    // Apply Z-position for 3D effect
                    if self.config.visualization_mode == VisualizationMode::ThreeD {
                        let z_offset = (layer_idx as f32 - node_indices.len() as f32 / 2.0) * self.config.z_spacing + self.config.layer_offset;
                        node.display_mut().z_pos = z_offset;
                        
                        // Apply 3D projection to node position
                        let original_pos = node.location();
                        let projected = project_3d_to_2d(
                            original_pos,
                            pivot,
                            z_offset,
                            self.config.rotation_x,
                            self.config.rotation_y,
                            self.config.rotation_z,
                            self.config.perspective_strength,
                        );
                        node.set_location(projected);
                    } else {
                        node.display_mut().z_pos = 0.0;
                    }
                }
            }
            
            let edge_indices: Vec<_> = self.graph.g().edge_indices().collect();
            for edge_idx in edge_indices {
                if let Some(edge) = self.graph.edge_mut(edge_idx) {
                    edge.display_mut().edge_color = self.config.edge_color;
                    edge.display_mut().selected_color = self.config.edge_selected_color;
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                let mut active_view_rect: Option<egui::Rect> = None;
                if self.current_tab == AppTab::NeuralNetwork {
                    // Render neural network
                    if let Some(ref mut nn_graph) = self.neural_network {
                        // Update show_values setting for all neural network nodes
                        let node_indices: Vec<_> = nn_graph.g().node_indices().collect();
                        for idx in node_indices {
                            if let Some(node) = nn_graph.node_mut(idx) {
                                node.display_mut().show_values = self.config.nn_config.show_neuron_values;
                            }
                        }
                        
                        let settings_interaction = SettingsInteraction::new()
                            .with_dragging_enabled(false)
                            .with_hover_enabled(true)
                            .with_node_clicking_enabled(false)
                            .with_node_selection_enabled(false);

                        let settings_navigation = SettingsNavigation::new()
                            .with_zoom_and_pan_enabled(true)
                            .with_fit_to_screen_enabled(self.config.fit_to_screen_enabled)
                            .with_zoom_speed(self.config.zoom_speed);

                        let settings_style = SettingsStyle::new()
                            .with_labels_always(false);
                        
                        let response = ui.add(
                            &mut GraphView::<_, _, _, _, NeuronNode, SynapseEdge>::new(nn_graph)
                                .with_id(Some(NEURAL_VIEW_ID.to_string()))
                                .with_interactions(&settings_interaction)
                                .with_navigations(&settings_navigation)
                                .with_styles(&settings_style),
                        );
                        active_view_rect = Some(response.rect);
                        
                        // Reset fit_to_screen after counter expires
                        if self.config.fit_to_screen_enabled && self.fit_to_screen_counter > 0 {
                            self.fit_to_screen_counter -= 1;
                            if self.fit_to_screen_counter == 0 {
                                self.config.fit_to_screen_enabled = false;
                            }
                            ctx.request_repaint();
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Neural network not initialized. Configure and generate in the sidebar.");
                        });
                    }
                } else {
                    // Render main graph
                    // Control simulation based on user setting - do this FIRST
                    let mut layout_state: FruchtermanReingoldState = get_layout_state(ui, None);
                    layout_state.is_running = self.config.simulation_running;
                    set_layout_state(ui, layout_state, None);
                    
                    let settings_interaction = SettingsInteraction::new()
                        .with_dragging_enabled(self.config.dragging_enabled)
                        .with_hover_enabled(self.config.hover_enabled)
                        .with_node_clicking_enabled(self.config.node_clicking_enabled)
                        .with_node_selection_enabled(self.config.node_selection_enabled)
                        .with_node_selection_multi_enabled(self.config.node_selection_multi_enabled)
                        .with_edge_clicking_enabled(self.config.edge_clicking_enabled)
                        .with_edge_selection_enabled(self.config.edge_selection_enabled)
                        .with_edge_selection_multi_enabled(self.config.edge_selection_multi_enabled);

                    let settings_navigation = SettingsNavigation::new()
                        .with_zoom_and_pan_enabled(self.config.zoom_and_pan_enabled)
                        .with_fit_to_screen_enabled(self.config.fit_to_screen_enabled)
                        .with_fit_to_screen_padding(self.config.fit_to_screen_padding)
                        .with_zoom_speed(self.config.zoom_speed);

                    let settings_style = SettingsStyle::new()
                        .with_labels_always(self.config.labels_always);
                    
                    let mut graph_view = GraphView::<_, _, _,_, CodeNode, CodeEdge>::new(&mut self.graph)
                        .with_id(Some(GRAPH_VIEW_ID.to_string()))
                        .with_interactions(&settings_interaction)
                        .with_navigations(&settings_navigation)
                        .with_styles(&settings_style);
                    
                    let response = ui.add(&mut graph_view);
                    active_view_rect = Some(response.rect);
                    
                    // Handle node double-click for text editing
                    // Check if double-clicked and find which node is hovered
                    if response.double_clicked() {
                        // Find the hovered node
                        for idx in self.graph.g().node_indices() {
                            if let Some(node) = self.graph.node(idx) {
                                if node.hovered() {
                                    // Start editing this node
                                    if let Some(class_info) = self.class_details.get(&idx) {
                                        self.editing_node = Some(idx);
                                        self.edit_text = class_info.name.clone();
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    
                    // Reset fit_to_screen after counter expires
                    if self.config.fit_to_screen_enabled && self.fit_to_screen_counter > 0 {
                        self.fit_to_screen_counter -= 1;
                        if self.fit_to_screen_counter == 0 {
                            self.config.fit_to_screen_enabled = false;
                        }
                        ctx.request_repaint();
                    }
                }
                
                // Draw grid and axes as overlay on top of graph
                let overlay_view_id = match self.current_tab {
                    AppTab::Graph => Some(GRAPH_VIEW_ID),
                    AppTab::NeuralNetwork if self.neural_network.is_some() => Some(NEURAL_VIEW_ID),
                    _ => None,
                };
                let overlay_rect = if overlay_view_id.is_some() {
                    active_view_rect
                } else {
                    None
                };
                self.draw_grid_and_axes(ui, overlay_view_id, overlay_rect);
                
                // Draw info overlay
                let painter = ui.painter();
                let rect = ui.max_rect();
                
                if self.current_tab == AppTab::Graph {
                    let node_count = self.graph.g().node_count();
                    let edge_count = self.graph.g().edge_count();
                    
                    let info_text = format!("Graph: {} nodes, {} edges | Zoom & Pan enabled | Click Center Graph to fit", node_count, edge_count);
                    painter.text(
                        Pos2::new(rect.min.x + 10.0, rect.min.y + 10.0),
                        egui::Align2::LEFT_TOP,
                        info_text,
                        egui::FontId::proportional(14.0),
                        Color32::from_rgb(180, 180, 180),
                    );
                } else if self.current_tab == AppTab::NeuralNetwork {
                    if self.neural_network.is_some() {
                        let info_text = "Neural Network | Use mouse to pan";
                        painter.text(
                            Pos2::new(rect.min.x + 10.0, rect.min.y + 10.0),
                            egui::Align2::LEFT_TOP,
                            info_text,
                            egui::FontId::proportional(14.0),
                            Color32::from_rgb(180, 180, 180),
                        );
                    }
                }

                // Handle node text editing overlay
                if self.current_tab == AppTab::Graph {
                    if let Some(editing_idx) = self.editing_node {
                        // Get node position and transform to screen coordinates
                        if let Some(node) = self.graph.node(editing_idx) {
                            let node_pos = node.location();
                            
                            // Get metadata for coordinate transformation
                            let meta = MetadataFrame::new(Some(GRAPH_VIEW_ID.to_string())).load(ui);
                            let screen_pos = meta.canvas_to_screen_pos(node_pos);
                            let node_radius = meta.canvas_to_screen_size(30.0);
                            
                            // Track whether to finish editing
                            let mut finish_editing = false;
                            let mut save_changes = false;
                            
                            // Calculate text width for centering
                            let text_width = (self.edit_text.len() as f32 * 8.0).max(80.0).min(200.0);
                            
                            // Create inline text edit centered on the node
                            egui::Area::new(egui::Id::new("inline_node_edit"))
                                .fixed_pos(Pos2::new(screen_pos.x - text_width / 2.0 - 12.0, screen_pos.y - 12.0))
                                .order(egui::Order::Foreground)
                                .show(ctx, |ui| {
                                    // Solid background frame that covers the node text
                                    egui::Frame::new()
                                        .fill(Color32::from_rgb(50, 50, 60))
                                        .stroke(Stroke::new(2.0, Color32::from_rgb(100, 180, 255)))
                                        .corner_radius(6.0)
                                        .inner_margin(egui::Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            let text_edit = egui::TextEdit::singleline(&mut self.edit_text)
                                                .desired_width(text_width)
                                                .font(egui::FontId::proportional(14.0))
                                                .text_color(Color32::WHITE)
                                                .horizontal_align(egui::Align::Center)
                                                .cursor_at_end(true);
                                            
                                            let response = ui.add(text_edit);
                                            
                                            // Auto-focus on the text input
                                            if !response.has_focus() {
                                                response.request_focus();
                                            }
                                            
                                            // Handle Enter to confirm
                                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                                save_changes = true;
                                                finish_editing = true;
                                            }
                                            
                                            // Handle Escape to cancel
                                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                                finish_editing = true;
                                            }
                                        });
                                });
                            
                            // Apply changes outside the closure to avoid borrow conflicts
                            if save_changes {
                                let new_name = self.edit_text.clone();
                                
                                // Update class_details (for sidebar)
                                if let Some(class_info) = self.class_details.get_mut(&editing_idx) {
                                    class_info.name = new_name.clone();
                                }
                                
                                // Update the node's payload in the graph - this is the source of truth
                                // The DisplayNode::update() method reads from payload.name
                                if let Some(node) = self.graph.node_mut(editing_idx) {
                                    node.payload_mut().name = new_name;
                                }
                            }
                            if finish_editing {
                                self.editing_node = None;
                            }
                        } else {
                            // Node no longer exists, cancel editing
                            self.editing_node = None;
                        }
                    }
                }
                
                // Show popup for hovered or selected nodes
                if self.current_tab == AppTab::Graph {
                    // Find hovered node
                    let mut hovered_node = None;
                    for idx in self.graph.g().node_indices() {
                        if let Some(node) = self.graph.node(idx) {
                            if node.hovered() {
                                hovered_node = Some(idx);
                                break;
                            }
                        }
                    }
                    self.hovered_node = hovered_node;
                    
                    // Find selected nodes
                    let mut selected_nodes: Vec<NodeIndex<u32>> = Vec::new();
                    for idx in self.graph.g().node_indices() {
                        if let Some(node) = self.graph.node(idx) {
                            if node.selected() {
                                selected_nodes.push(idx);
                            }
                        }
                    }
                    
                    // Show popup for hovered node (priority)
                    if let Some(node_idx) = self.hovered_node {
                        self.draw_hover_popup(ui, node_idx);
                    }
                    
                    // Show popups for selected nodes (if not already showing hovered)
                    for selected_idx in selected_nodes {
                        // Don't show duplicate popup if this node is also hovered
                        if self.hovered_node != Some(selected_idx) {
                            self.draw_hover_popup(ui, selected_idx);
                        }
                    }
                }
            });
        }
        
        fn save(&mut self, storage: &mut dyn eframe::Storage) {
            eframe::set_value(storage, "app_config", &self.config);
            eframe::set_value(storage, "auto_save_config", &self.auto_save_config);
        }
    }
}
