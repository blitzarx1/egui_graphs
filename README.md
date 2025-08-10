![build](https://github.com/blitzarx1/egui_graphs/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs

Graph visualization with rust, [petgraph](https://github.com/petgraph/petgraph) and [egui](https://github.com/emilk/egui) in its DNA.

<img width="1011" height="642" alt="Screenshot 2025-08-10 at 10 41 35 PM" src="https://github.com/user-attachments/assets/4b55f220-8b25-4809-8bf2-44452a77f374" />

The project implements a Widget for the egui framework, enabling easy visualization of interactive graphs in rust. The goal is to implement the very basic engine for graph visualization within egui, which can be easily extended and customized for your needs.

- [x] Visualization of any complex graphs;
- [x] Layots and custom layout mechanism;
- [x] Zooming and panning;
- [x] Node and edges interactions and events reporting: click, double click, select, drag;
- [x] Node and Edge labels;
- [x] Dark/Light theme support via egui context styles;
- [x] User stroke styling hooks (node & edge) for dynamic customization;

## Status

The project is on track for a stable release v1.0.0. For the moment, breaking releases are very possible.

Please use master branch for the latest updates.

Check the [demo example](https://github.com/blitzar-tech/egui_graphs/blob/main/examples/demo.rs) for the comprehensive overview of the widget possibilities.

## Layouts

In addition to the basic graph display functionality, the project provides a layout mechanism to arrange the nodes in the graph. The `Layout` trait can be implemented by the library user allowing for custom layouts. The following layouts are provided out of the box:

- [x] Random layout;
- [x] Hierarchical layout;
- [x] Force-directed layout (naive baseline);

<img width="441" height="425" alt="Screenshot 2025-08-10 at 1 38 45 PM" src="https://github.com/user-attachments/assets/aebcb954-4ce8-4492-81be-2a787dfcdba8" />
<img width="441" height="425" alt="Screenshot 2025-08-10 at 1 38 45 PM" src="https://github.com/user-attachments/assets/48614f43-4436-42eb-a238-af196d2044b4" />

Check the [layouts example](https://github.com/blitzarx1/egui_graphs/blob/master/examples/layouts/src/main.rs).

### Force-directed layout

A naive O(n²) force-directed layout (Fruchterman–Reingold style) is now included. It exposes adjustable simulation parameters (step size, damping, gravity, etc.). For a live demonstration and tuning panel, see the [demo example](https://github.com/blitzar-tech/egui_graphs/blob/main/examples/demo.rs). This implementation is a baseline and may be optimized in future releases.

## Examples

### Basic setup example

The source code of the following steps can be found in the [basic example](https://github.com/blitzarx1/egui_graphs/blob/master/examples/basic/src/main.rs).

#### Step 1: Setting up the `BasicApp` struct

First, let's define the `BasicApp` struct that will hold the graph.

```rust
pub struct BasicApp {
    g: egui_graphs::Graph,
}
```

#### Step 2: Implementing the `new()` function

Next, implement the `new()` function for the `BasicApp` struct.

```rust
impl BasicApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: egui_graphs::Graph::from(&g) }
    }
}
```

#### Step 3: Generating the graph

Create a helper function called `generate_graph()`. In this example, we create three nodes and three edges.

```rust
fn generate_graph() -> petgraph::StableGraph<(), ()> {
    let mut g = petgraph::StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}
```

#### Step 4: Implementing the `eframe::App` trait

Now, lets implement the `eframe::App` trait for the `BasicApp`. In the `update()` function, we create a `egui::CentralPanel` and add the `egui_graphs::GraphView` widget to it.

```rust
impl eframe::App for BasicApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut egui_graphs::GraphView::new(&mut self.g));
        });
    }
}
```

#### Step 5: Running the application

Finally, run the application using the `eframe::run_native()` function.

```rust
fn main() {
    eframe::run_native(
        "egui_graphs_basic_demo",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
    .unwrap();
}
```

![Screenshot 2023-10-14 at 23 49 49](https://github.com/blitzarx1/egui_graphs/assets/32969427/584b78de-bca3-421b-b003-9321fd3e1b13)
You can further customize the appearance and behavior of your graph by modifying the settings or adding more nodes and edges as needed.

## Features

### Styling Hooks (Node & Edge Strokes)

You can now override the stroke style (width / color / alpha) used to draw nodes and edges without re-implementing the default display shapes. Provide closures via `SettingsStyle`:

```rust
let style = egui_graphs::SettingsStyle::new()
    .with_edge_stroke_hook(|selected, order, stroke, egui_style| {
        // Fade unselected edges, keep selected crisp; vary slightly by parallel edge order.
        let mut s = stroke;
        if !selected {
            let c = s.color;
            s.color = egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as f32 * 0.5) as u8);
        }
        // Subtle darkening for higher-order parallel edges
        let factor = 1.0 - (order as f32 * 0.08).min(0.4);
        s.color = s.color.linear_multiply(factor);
        s
    })
    .with_node_stroke_hook(|selected, dragged, node_color, stroke, egui_style| {
        let mut s = stroke;
        // Base color: explicit node color or egui visuals
        s.color = node_color.unwrap_or_else(|| egui_style.visuals.widgets.inactive.fg_stroke.color);
        if selected { s.width = 3.0; }
        if dragged { s.color = egui::Color32::LIGHT_BLUE; }
        s
    });

let mut view = egui_graphs::GraphView::new(&mut graph)
    .with_styles(&style);
```

Hooks receive the current `Stroke` derived from the active egui theme, so your custom logic stays consistent with light/dark modes.

#### Hooks vs. Implement `Display<Node|Edge> Trait`

Use a stroke hook when you only need quick visual tweaks (color / width / alpha) based on interaction state or simple heuristics.
Implement a custom `DisplayNode` / `DisplayEdge` when you need to change geometry (different shapes, icons, multiple layered outlines), custom hit‑testing, animations, or rich graph‑context dependent visuals.

| Need | Hook | Custom Drawer |
|------|------|---------------|
| Adjust stroke color/width on select/hover | ✅ | ✅ |
| Fade or highlight edges | ✅ | ✅ |
| Different node shape (rect, hex, image, pie) | ❌ | ✅ |
| Custom label placement / multiple labels | ❌ | ✅ |
| Custom hit area / hit test logic | ❌ | ✅ |
| Graph‑topology aware geometry (hub size, cluster halos) | ❌ | ✅ |
| Minimal boilerplate | ✅ | ❌ |

Rule of thumb: start with hooks; switch to a custom drawer if you find yourself wanting to modify anything beyond the single stroke per node/edge.

### Events

Can be enabled with `events` feature. Events describe a change made in graph whether it changed zoom level or node dragging.

Combining this feature with custom node draw function allows to implement custom node behavior and drawing according to the events happening.
