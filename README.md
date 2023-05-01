[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs
Graph visualization with rust, [petgraph](https://github.com/petgraph/petgraph) and [egui](https://github.com/emilk/egui) in its DNA.

![Screenshot 2023-04-28 at 23 14 38](https://user-images.githubusercontent.com/32969427/235233765-23b0673b-70e5-4138-9384-180804392dba.png)

## Features
* Arbitrarily complex graphs with self-references, multiple edges, loops, etc.;
* Zoom & pan;
* Dragging, Selecting;
* Graph elements style configuration;
* Support for egui dark/light mode;

## Status
The project is on the track to v1.0.0 and there will be some minor releases prior. So stay tuned!

### Basic setup example
#### Step 1: Setting up the BasicApp struct. 

First, let's define the `BasicApp` struct that will hold the graph.
```rust 
pub struct BasicApp {
    g: StableGraph<Node<()>, Edge<()>>,
}
```

#### Step 2: Implementing the new() function. 

Next, implement the `new()` function for the `BasicApp` struct.
```rust
impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g }
    }
}
```

#### Step 3: Generating the graph. 

Create a helper function called `generate_graph()`. In this example, we create three nodes with and three edges connecting them in a triangular pattern.
```rust 
fn generate_graph() -> StableGraph<Node<()>, Edge<()>> {
    let mut g: StableGraph<Node<()>, Edge<()>> = StableGraph::new();

    let a = g.add_node(Node::new(egui::Vec2::new(0., SIDE_SIZE), ()));
    let b = g.add_node(Node::new(egui::Vec2::new(-SIDE_SIZE, 0.), ()));
    let c = g.add_node(Node::new(egui::Vec2::new(SIDE_SIZE, 0.), ()));

    g.add_edge(a, b, Edge::new(()));
    g.add_edge(b, c, Edge::new(()));
    g.add_edge(c, a, Edge::new(()));

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

![Screenshot 2023-04-24 at 22 04 49](https://user-images.githubusercontent.com/32969427/234086555-afdf5dfa-31be-46f2-b46e-1e9a45e1a50f.png)


You can further customize the appearance and behavior of your graph by modifying the settings or adding more nodes and edges as needed.

### Interactive

You can check more advanced [configurable demo](https://github.com/blitzarx1/egui_graph/tree/master/examples/configurable) for usage references and settings description.

## Gallery
![ezgif-4-3e4e4469e6](https://user-images.githubusercontent.com/32969427/233863786-11459176-b741-4343-8b42-7d9b3a8239ee.gif)

![dynamic_demo](https://user-images.githubusercontent.com/32969427/235311610-b59b4cfb-3e93-49a2-8780-61a83a95af03.gif)
