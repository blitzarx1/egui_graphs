![build](https://github.com/blitzarx1/egui_graphs/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs

Graph visualization with rust, [petgraph](https://github.com/petgraph/petgraph) and [egui](https://github.com/emilk/egui) in its DNA.

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

You can implement your own force by implementing the `ExtraForce` trait and then composing it via `Extra<MyExtra, ENABLED>` in a tuple. To keep this README focused, see the trait docs for a full example and method signature (docs.rs → egui_graphs → layouts → force_directed → extras → core → ExtraForce).

Once implemented, use the public aliases to plug it in:

```rust
use egui_graphs::{Extra, FruchtermanReingoldWithExtras, FruchtermanReingoldWithExtrasState, LayoutForceDirected};

type Extras = (Extra<MyExtra, true>, ());
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

With the events feature enabled:

```bash
cd crates/demo-web
trunk serve --features events
```

Build static assets:

```bash
cd crates/demo-web
trunk build
# output in crates/demo-web/dist
```

Build with the events feature enabled:

```bash
cd crates/demo-web
trunk build --features events
```

### Run any native example

From the workspace root, specify the package and the example name:

```bash
# demo example
cargo run -p egui_graphs --example demo

# another example (basic)
cargo run -p egui_graphs --example basic

# inspect per-frame GraphView responses
cargo run -p egui_graphs --example graph_view_response

# enable features (e.g., events)
cargo run -p egui_graphs --example demo --features events

# release mode
cargo run -p egui_graphs --example demo --release
```
