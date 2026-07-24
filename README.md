![build](https://github.com/blitzarx1/egui_graphs/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs

Graph visualization with rust and [egui](https://github.com/emilk/egui).

![ezgif-7312f131a6515c6e](https://github.com/user-attachments/assets/22d8ce17-be22-4dc5-a337-4cea795cf46c)

The project provides a `GraphView` for the egui framework, enabling easy visualization of interactive graphs in rust. The goal is to implement the very basic engine for graph visualization within egui, which can be easily extended and customized for your needs.

Check the [web-demo](https://blitzarx1.github.io/egui_graphs) for the comprehensive overview of the widget possibilities.

- [x] Build wasm or native;
- [x] Layouts and custom layout mechanism;
- [x] Zooming and panning;
- [x] Node and edge interaction reporting: click, double click, select, hover, drag;
- [x] Node and Edge labels;
- [x] Dark/Light theme support via egui context styles;
- [x] User stroke styling hooks (node & edge) for dynamic customization;

## Table of Contents

- [Status](#status)
- [Examples](#examples)
- [Features](#features)
  - [Layouts](#layouts)
  - [GraphView response](#graphview-response)
- [Repository organization](#repository-organization)
- [Run locally](#run-locally)
  - [Run the web demo (WASM)](#run-the-web-demo-wasm)
  - [Run any native example](#run-any-native-example)

## Status

*The project is not in active development. Feel free to fork it and tweak for your needs.*

## Examples

### Basic setup example

The source code of the following steps can be found in the [basic example](https://github.com/blitzarx1/egui_graphs/blob/main/crates/egui_graphs/examples/basic.rs).

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
        Self { g: generate_graph() }
    }
}
```

#### Step 3: Generating the graph

Create a helper function called `generate_graph()`. In this example, we create three nodes and three edges.

```rust
fn generate_graph() -> egui_graphs::Graph {
    let mut g = egui_graphs::Graph::new();

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

Now, lets implement the `eframe::App` trait for the `BasicApp`. In the `ui()` function, we create an `egui::CentralPanel` and show the `egui_graphs::GraphView` in it.

```rust
impl eframe::App for BasicApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui_graphs::DefaultGraphView::new().show(ui, &mut self.g);
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

![Basic example screenshot](https://github.com/user-attachments/assets/be2eaf6c-0c88-4450-9825-2d7640278d7f)

You can further customize the appearance and behavior of your graph by modifying the settings or adding more nodes and edges as needed.

### Custom renderers

Custom `DisplayNode` and `DisplayEdge` implementations receive a `DrawContext`. Use
`ctx.style.labels_always()`, `ctx.style.resolve_node_stroke(...)`, and
`ctx.style.resolve_edge_stroke(...)` inside those renderers to preserve the graph-wide label and
stroke settings. The public `EdgeShapeBuilder`, `EdgeShapeProps`, and `TipProps` types can be used
to construct the same straight, curved, looped, and arrow-tipped edge geometry as the default
renderer without copying its internals.

## Features

### Layouts

Built-in layouts with a pluggable API. The `Layout` trait powers layout selection and persistence; you can plug different algorithms or implement your own.

- Random: quick scatter for any graph (default via `DefaultGraphView`).
- Hierarchical: layered (ranked) layout.
- Force-directed: Fruchterman–Reingold baseline with optional Extras (e.g., Center Gravity).

#### Quick start

```rust
// Default random layout
egui_graphs::DefaultGraphView::new().show(ui, &mut graph);

// Pick a specific layout (Hierarchical)
type L = egui_graphs::LayoutHierarchical;
type S = egui_graphs::LayoutStateHierarchical;
egui_graphs::GraphView::<S, L>::new().show(ui, &mut graph);

// Force‑Directed (FR) with Center Gravity
type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;
egui_graphs::GraphView::<S, L>::new().show(ui, &mut graph);
```

#### In-depth: Force‑Directed layout

A naive O(n²) force-directed layout (Fruchterman–Reingold style) is included. It exposes adjustable simulation parameters (step size, damping, etc.). See the demo for a live tuning panel. Built-in options include the baseline Fruchterman–Reingold and an extended variant with composable “extras” (e.g., Center Gravity).

Select algorithm via the layout type parameter (public aliases):

```rust
use egui_graphs::{LayoutForceDirected, FruchtermanReingold, FruchtermanReingoldState};

type L = LayoutForceDirected<FruchtermanReingold>;
type S = FruchtermanReingoldState;
egui_graphs::GraphView::<S, L>::new().show(ui, &mut graph);
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
let mut state = egui_graphs::get_layout_state::<S>(ui, None);
state.base.is_running = true;
state.extras.0.params.c = 0.2;
egui_graphs::set_layout_state(ui, state, None);
egui_graphs::GraphView::<S, L>::new().show(ui, &mut graph);
```

##### Author a custom extra

Implement `ExtraForce` and compose it through `Extra<MyExtra, ENABLED>` in a tuple. Extra-force parameters are persisted with layout state, so they must implement `Serialize` and `Deserialize`.
The complete runnable version is in the [`custom_force` example](https://github.com/blitzarx1/egui_graphs/blob/main/crates/egui_graphs/examples/custom_force.rs).

```rust
use egui::{Rect, Vec2};
use egui_graphs::{
    DisplayEdge, DisplayNode, Extra, ExtraForce, FruchtermanReingoldWithExtras,
    FruchtermanReingoldWithExtrasState, Graph, LayoutForceDirected,
};
use petgraph::EdgeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
struct PullToCenter;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct PullToCenterParams {
    strength: f32,
}

impl ExtraForce for PullToCenter {
    type Params = PullToCenterParams;

    fn apply<N, E, Ty, Ix, Dn, De>(
        params: &Self::Params,
        graph: &Graph<N, E, Ty, Ix, Dn, De>,
        indices: &[petgraph::stable_graph::NodeIndex<Ix>],
        displacements: &mut [Vec2],
        area: Rect,
        _k: f32,
    ) where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        for (position, index) in indices.iter().enumerate() {
            let location = graph.g().node_weight(*index).unwrap().location();
            displacements[position] += (area.center() - location) * params.strength;
        }
    }
}

type Extras = (Extra<PullToCenter, true>, ());
type S = FruchtermanReingoldWithExtrasState<Extras>;
type L = LayoutForceDirected<FruchtermanReingoldWithExtras<Extras>>;
egui_graphs::GraphView::<S, L>::new().show(ui, &mut graph);
```

Composition is order-sensitive; each enabled extra accumulates into the shared displacement vector in tuple order.

### GraphView response

`GraphView::show` returns a `GraphViewResponse`. Its `response` field is the standard `egui::Response` for the allocated graph area, while `changes` contains the graph-specific changes produced by that call:

```rust
let result = egui_graphs::DefaultGraphView::new().show(ui, &mut graph);

if result.response.hovered() {
    // The pointer is over this GraphView.
}

for change in result.changes {
    println!("{change:?}");
}
```

Changes are transient and ordered by occurrence. Repeated changes are kept as separate entries, and an interaction-free frame returns an empty vector. Variants involving nodes or edges use the graph's `NodeIndex<Ix>` or `EdgeIndex<Ix>` type; graph-wide changes such as pan and zoom do not depend on an entity index. Store or aggregate a batch in your application when it must outlive the current frame. See the [`graph_view_response` example](https://github.com/blitzarx1/egui_graphs/blob/main/crates/egui_graphs/examples/graph_view_response.rs) for a complete application.

## Repository organization

Crates:

- crates/egui_graphs – library crate published to crates.io
- crates/demo-core – shared demo logic (not published)
- crates/demo-web – WASM web demo (not published)

Build from the workspace root:

```bash
cargo build --workspace
```

## Run locally

### Run the web demo (WASM)

Prerequisites:

- Rust toolchain
- wasm target and trunk

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk # if not installed
```

Serve locally:

```bash
cd crates/demo-web
trunk serve
# opens http://127.0.0.1:8080 (or similar)
```

Build static assets:

```bash
cd crates/demo-web
trunk build
# output in crates/demo-web/dist
```

### Run any native example

From the workspace root, specify the package and the example name:

```bash
# demo example
cargo r --release --example demo

# another example (basic)
cargo r --release --example basic

# custom force-directed layout
cargo r --release --example custom_force

# inspect per-frame GraphView responses
cargo r --release --example graph_view_response
```
