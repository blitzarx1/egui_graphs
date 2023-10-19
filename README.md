![build](https://github.com/blitzarx1/egui_graphs/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs
Graph visualization with rust, [petgraph](https://github.com/petgraph/petgraph) and [egui](https://github.com/emilk/egui) in its DNA.

![Screenshot 2023-04-28 at 23 14 38](https://user-images.githubusercontent.com/32969427/235233765-23b0673b-70e5-4138-9384-180804392dba.png)

The project implements a Widget for the egui framework, enabling easy visualization of interactive graphs in rust. The goal is to implement the very basic engine for graph visualization within egui, which can be easily extended and customized for your needs.

- [x] Visualization of any complex graphs;
- [x] Zooming and panning;
- [x] Node labels;
- [x] Node interactions and events reporting: click, double click, select, drag;
- [x] Style configuration via egui context styles;
- [x] Dark/Light theme support via egui context styles;
- [x] Events reporting to extend the graph functionality by the user handling them;

## Status
The project is on track for a stable release v1.0.0. For the moment, breaking releases are still possible.

## Features
### Events
Can be enabled with `events` feature.
- [x] Node click;
- [x] Node double click;
- [x] Node select;
- [x] Node move;
- [x] Node drag;
- [ ] Node hover;

Combining this feature with custom node draw function allows to implement custom node behavior and drawing according to the events happening.

## Egui crates features support
### Persistence
To use egui `persistence` feature you need to enable `egui_persistence` feature of this crate. For example:
```toml
egui_graphs = { version = "0", features = ["egui_persistence"]}
egui = {version="0.23", features = ["persistence"]}
```

## Examples
### Basic setup example
#### Step 1: Setting up the BasicApp struct. 
First, let's define the `BasicApp` struct that will hold the graph.
```rust 
pub struct BasicApp {
    g: Graph<(), (), Directed>,
}
```

#### Step 2: Implementing the new() function. 
Next, implement the `new()` function for the `BasicApp` struct.
```rust
impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}
```

#### Step 3: Generating the graph. 
Create a helper function called `generate_graph()`. In this example, we create three nodes with and three edges connecting them in a triangular pattern.
```rust 
fn generate_graph() -> StableGraph<(), (), Directed> {
    let mut g: StableGraph<(), ()> = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}
```

#### Step 4: Implementing the update() function. 
Now, lets implement the `update()` function for the `BasicApp`. This function creates a `GraphView` widget providing a mutable reference to the graph, and adds it to `egui::CentralPanel` using the `ui.add()` function for adding widgets.
```rust 
impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::new(&mut self.g));
        });
    }
}
```

#### Step 5: Running the application. 
Finally, run the application using the `run_native()` function with the specified native options and the `BasicApp`.
```rust 
fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Box::new(BasicApp::new(cc))),
    )
    .unwrap();
}
```

![Screenshot 2023-10-14 at 23 49 49](https://github.com/blitzarx1/egui_graphs/assets/32969427/584b78de-bca3-421b-b003-9321fd3e1b13)
You can further customize the appearance and behavior of your graph by modifying the settings or adding more nodes and edges as needed.
