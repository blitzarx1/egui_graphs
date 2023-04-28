[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs
Graph visualization with rust and [egui](https://github.com/emilk/egui)

![Screenshot 2023-04-28 at 23 14 38](https://user-images.githubusercontent.com/32969427/235233765-23b0673b-70e5-4138-9384-180804392dba.png)

## Features
* Customization and interactivity;
* Ability to draw arbitrarily complex graphs with self-references, loops, etc.;
* Zoom & pan;
* Dragging, Selecting;
* The `GraphView` widget does not modify the provided graph and properties, instead in case of any interactions, it generates changes which the client can apply;
* Support for egui dark/light mode;

![ezgif-4-3e4e4469e6](https://user-images.githubusercontent.com/32969427/233863786-11459176-b741-4343-8b42-7d9b3a8239ee.gif)

---
## Applying changes from GraphView widget

The `GraphView` widget provides a way to visualize a graph and interact with it by dragging nodes, selecting nodes and edges, and more. However, in order to update the underlying graph data structure with these changes, we need to apply the changes returned by the widget.

This is where the `Elements` struct comes in. The `Elements` struct contains the graph data that is used to render the `GraphView` widget, and provides a method to apply the changes to this data.

The simplest way to apply changes is to call the `apply_changes` method on the `Elements` struct. This method accepts a `Changes` struct which contains information about the changes that were made in the `GraphView` widget, and applies these changes to the `Elements` struct.

Default operations for applying changes to `Elements` itself will be performed automatically when the method is called. User callback is needed to perform any additional actions which can be required by the user's application. This is a good place to sync changes to your graph backend for example petgraph.

```rust
elements.apply_changes(changes, &mut |elements, node_idx, change| {
  // some additional manipulation with changed node using its idx `node_idx`
  // or manipulations with other elements via mutable reference to `elements`
  
  println!("changes for node {} applied", node_idx);
})
```

---

### Basic setup example
#### Step 1: Setting up the BasicApp struct. 

First, let's define the `BasicApp` struct that will hold the graph elements and settings. The struct contains two fields: `elements` and `settings`. The elements field stores the graph's nodes and edges, while settings contains the configuration options for the GraphView widget.
```rust 
pub struct BasicApp {
    elements: Elements,
}
```

#### Step 2: Implementing the new() function. 

Next, implement the `new()` function for the `BasicApp` struct. This function initializes the graph settings with default values and generates the graph elements.
```rust
impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let elements = generate_graph();
        Self { elements }
    }
}
```

#### Step 3: Generating the graph elements. 

Create a helper function called `generate_graph()` that initializes the nodes and edges for the graph. In this example, we create three nodes with unique positions and three edges connecting them in a triangular pattern.
```rust 
fn generate_graph() -> Elements {
    let mut nodes = HashMap::new();
    nodes.insert(0, Node::new(0, egui::Vec2::new(0., SIDE_SIZE)));
    nodes.insert(1, Node::new(1, egui::Vec2::new(-SIDE_SIZE, 0.)));
    nodes.insert(2, Node::new(2, egui::Vec2::new(SIDE_SIZE, 0.)));

    let mut edges = HashMap::new();
    edges.insert((0, 1), vec![Edge::new(0, 1, 0)]);
    edges.insert((1, 2), vec![Edge::new(1, 2, 0)]);
    edges.insert((2, 0), vec![Edge::new(2, 0, 0)]);

    Elements::new(nodes, edges)
}
```

#### Step 4: Implementing the update() function. 

Now, implement the `update()` function for the `BasicApp`. This function creates a `GraphView` widget with the `elements`, and adds it to the central panel using the `ui.add()` function.
```rust 
impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(GraphView::new(&self.elements));
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


You can further customize the appearance and behavior of your graph by modifying the settings or adding more nodes and edges as needed. Don't forget to apply changes returned from the widget.

### Interactive

You can check more advanced [interactive example](https://github.com/blitzarx1/egui_graph/tree/master/examples/interactive) for usage references, settings description and changes apply demonstration.
