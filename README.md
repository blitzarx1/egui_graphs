This repository has been reorganized into a Cargo workspace with a `crates/` layout similar to the `egui` monorepo.

Crates:
- crates/egui_graphs – library crate published to crates.io
- crates/demo-core – shared demo logic (not published)
- crates/demo-web – WASM web demo (not published)

Build from the workspace root:

```
cargo build --workspace
```
![build](https://github.com/blitzarx1/egui_graphs/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs

Graph visualization with rust, [petgraph](https://github.com/petgraph/petgraph) and [egui](https://github.com/emilk/egui) in its DNA.

![ezgif-782ac39a721d13](https://github.com/user-attachments/assets/56e6f244-ce8f-48f4-b269-681e266c365f)

The project implements a Widget for the egui framework, enabling easy visualization of interactive graphs in rust. The goal is to implement the very basic engine for graph visualization within egui, which can be easily extended and customized for your needs.

- [x] Visualization of any complex graphs;
- [x] Layouts and custom layout mechanism;
- [x] Zooming and panning;
- [x] Node and edges interactions and events reporting: click, double click, select, drag;
- [x] Node and Edge labels;
- [x] Dark/Light theme support via egui context styles;
- [x] User stroke styling hooks (node & edge) for dynamic customization;

## Status

The project is on track for a stable release v1.0.0. For the moment, breaking releases are very possible.

Please use `main` branch for the latest updates.

Check the [demo example](https://github.com/blitzar-tech/egui_graphs/blob/main/examples/demo.rs) for the comprehensive overview of the widget possibilities.

## Examples

### Basic setup example

The source code of the following steps can be found in the [basic example](https://github.com/blitzar-tech/egui_graphs/blob/main/examples/basic.rs).

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

<img width="792" height="594" alt="Screenshot 2025-08-16 at 7 46 25 PM" src="https://github.com/user-attachments/assets/be2eaf6c-0c88-4450-9825-2d7640278d7f" />

You can further customize the appearance and behavior of your graph by modifying the settings or adding more nodes and edges as needed.

## Features

### Layouts

Built-in layouts with a pluggable API. The `Layout` trait powers layout selection and persistence; you can plug different algorithms or implement your own.

- Random: quick scatter for any graph (default via `DefaultGraphView`).
- Hierarchical: layered (ranked) layout.
- Force-directed: Fruchterman–Reingold baseline with optional Extras (e.g., Center Gravity).

#### Quick start

```rust
// Default random layout
let mut view = egui_graphs::DefaultGraphView::new(&mut graph);
ui.add(&mut view);

// Pick a specific layout (Hierarchical)
type L = egui_graphs::LayoutHierarchical;
type S = egui_graphs::LayoutStateHierarchical;
let mut view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut graph);
ui.add(&mut view);

// Force‑Directed (FR) with Center Gravity
type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;
let mut view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut graph);
ui.add(&mut view);
```

#### In-depth: Force‑Directed layout

A naive O(n²) force-directed layout (Fruchterman–Reingold style) is included. It exposes adjustable simulation parameters (step size, damping, etc.). See the demo for a live tuning panel. Built-in options include the baseline Fruchterman–Reingold and an extended variant with composable “extras” (e.g., Center Gravity).

Select algorithm via the layout type parameter (public aliases):

```rust
use egui_graphs::{LayoutForceDirected, FruchtermanReingold, FruchtermanReingoldState};

type L = LayoutForceDirected<FruchtermanReingold>;
type S = FruchtermanReingoldState;
let mut view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut graph);
```

#### Extras (composable add‑ons)

Use `FruchtermanReingoldWithExtras<E>` to apply base FR forces plus your extras each frame. Built-in extra: Center Gravity.

```rust
use egui_graphs::{
    LayoutForceDirected,
    FruchtermanReingoldWithCenterGravity,
    FruchtermanReingoldWithCenterGravityState,
};

type L = LayoutForceDirected<FruchtermanReingoldWithCenterGravity>;
type S = FruchtermanReingoldWithCenterGravityState;
let mut state = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::get_layout_state(ui);
state.base.is_running = true;
state.extras.0.params.c = 0.2;
egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::set_layout_state(ui, state);
let mut view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut graph);
ui.add(&mut view);
```

##### Author a custom extra

You can implement your own force by implementing the `ExtraForce` trait and then composing it via `Extra<MyExtra, ENABLED>` in a tuple. To keep this README focused, see the trait docs for a full example and method signature (docs.rs → egui_graphs → layouts → force_directed → extras → core → ExtraForce).

Once implemented, use the public aliases to plug it in:

```rust
use egui_graphs::{Extra, FruchtermanReingoldWithExtras, FruchtermanReingoldWithExtrasState, LayoutForceDirected};

type Extras = (Extra<MyExtra, true>, ());
type S = FruchtermanReingoldWithExtrasState<Extras>;
type L = LayoutForceDirected<FruchtermanReingoldWithExtras<Extras>>;
let mut view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut graph);
```

Composition is order-sensitive; each enabled extra accumulates into the shared displacement vector in tuple order.

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
