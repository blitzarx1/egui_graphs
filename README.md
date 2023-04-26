[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs
Grpah visualization implementation using [egui](https://github.com/emilk/egui)

![Screenshot 2023-04-23 at 22 02 42](https://user-images.githubusercontent.com/32969427/233856916-4b3cf1a7-85a3-4ca4-8d07-bac9fd0d95d6.png)

## Status
The project is close to the first stable version.

Currently not optimized for large graphs. The goal is to match egui drawing speed. Further optimizations are unnecessary.

## Concept
The goal is to create a crate that expands egui's visualization capabilities and offers an easy-to-integrate, customizable graph visualization widget.

* Customization and interactivity;
* Ability to draw arbitrarily complex graphs with self-references, loops, etc.;
* Widget does not modify the provided graph and properties; instead, it generates changes, in case of any interactions, which the client can apply.

## Roadmap for v0.1.0 - first stable release
<pre>
                                      done
-------------------------------------+----
basic graph drawing                  | [x]
self-references, multi-connections   | [x]
zoom & pan, fit-to-screen            | [x]
drag node                            | [x]
select, multiselect.                 | [x]
style customizations                 | [ ]
support egui dark/light theme        | [ ]
interactions vs egui draw benchmarks | [ ]
documentation, tests, example        | [ ]
</pre>

---
## Applying changes from GraphView widget

The `GraphView` widget in the egui_graphs crate provides a way to visualize a graph and interact with it by dragging nodes, selecting nodes and edges, and more. However, in order to update the underlying graph data structure with these changes, we need to apply the changes returned by the widget after each frame.

This is where the `Elements` struct comes in. The `Elements` struct contains the graph data that is used to render the `GraphView` widget, and provides methods to apply changes to this data.

The simplest way to apply changes is to call the `apply_changes` method on the `Elements` struct. This method accepts a `Changes` struct which contains information about the changes that were made in the `GraphView` widget, and applies these changes to the `Elements` struct. For example:

```rust
let mut elements = construct_elements();
let changes = graph_view.last_changes();
elements.apply_changes(changes, &mut on_node_change, &mut on_edge_change);
```

In this example, `construct_elements()` is a function that creates an initial `Elements` struct, and `graph_view` is a reference to the `GraphView` widget. The `last_changes` method on the GraphView widget returns a `Changes` struct containing the changes made in the widget. Finally, the `apply_changes` method on the `Elements` struct applies these changes, calling the `on_node_change` and `on_edge_change` callbacks for each node and edge that was changed after default changes has been applied.

The `apply_changes` method is flexible and allows for custom behavior when changes are applied. For example, if you want to update some external data structure when a node is moved in the `GraphView` widget, you can provide a callback function to the `apply_changes` method:

```rust
fn on_node_change(node: &mut Node, change: &NodeChange) {
    if let Some(location_change) = change.location {
        // update external data structure with new location
        update_location(node.id, location_change.x, location_change.y);
    }
}

let mut elements = construct_elements();
let changes = graph_view.last_changes();
elements.apply_changes(changes, &mut on_node_change, &mut on_edge_change);
```

In this example, the `on_node_change` function is called for each node that was changed in the `GraphView` widget. If the `location` field in the `NodeChange` struct is present, the function updates some external data structure with the new location of the node.

By using the `apply_changes` method and providing custom callback functions, we can easily apply changes made in the `GraphView` widget to our graph data structure and perform any additional tasks we need to when changes are made.

---
## Examples

![ezgif-4-3e4e4469e6](https://user-images.githubusercontent.com/32969427/233863786-11459176-b741-4343-8b42-7d9b3a8239ee.gif)

### Basic
#### Step 1: Setting up the ExampleApp struct. 

First, let's define the `ExampleApp` struct that will hold the graph elements and settings. The struct contains two fields: `elements` and `settings`. The elements field stores the graph's nodes and edges, while settings contains the configuration options for the GraphView widget.
```rust 
pub struct ExampleApp {
    elements: Elements,
    settings: Settings,
}
```

#### Step 2: Implementing the new() function. 

Next, implement the `new()` function for the `ExampleApp` struct. This function initializes the graph settings with default values and generates the graph elements.
```rust
impl ExampleApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let settings = Settings::default();
        let elements = generate_graph();
        Self { settings, elements }
    }
}
```

#### Step 3: Generating the graph elements. 

Create a helper function called `generate_graph()` that initializes the nodes and edges for the graph. In this example, we create three nodes with unique positions and three edges connecting them in a triangular pattern.
```rust 
fn generate_graph() -> Elements {
    let mut nodes = HashMap::new();
    nodes.insert(0, Node::new(0, egui::Vec2::new(0., 30.)));
    nodes.insert(1, Node::new(1,egui::Vec2::new(-30., 0.)));
    nodes.insert(2, Node::new(2,egui::Vec2::new(30., 0.)));
    
    let mut edges = HashMap::new();
    edges.insert((0, 1), vec![Edge::new(0, 1, 0)]);
    edges.insert((1, 2), vec![Edge::new(1, 2, 0)]);
    edges.insert((2, 0), vec![Edge::new(2, 0, 0)]);   
    
    Elements::new(nodes, edges)
}
```

#### Step 4: Implementing the update() function. 

Now, implement the `update()` function for the `ExampleApp`. This function creates a `GraphView` widget with the `elements` and `settings`, and adds it to the central panel using the `ui.add()` function.
```rust 
impl App for ExampleApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        let widget = &GraphView::new(&self.elements, &self.settings);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(widget);
        });
    }
}
```

#### Step 5: Running the application. 

Finally, run the application using the `run_native()` function with the specified native options and the `ExampleApp`.
```rust 
fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Box::new(ExampleApp::new(cc))),
    )
    .unwrap();
}
```

![Screenshot 2023-04-24 at 22 04 49](https://user-images.githubusercontent.com/32969427/234086555-afdf5dfa-31be-46f2-b46e-1e9a45e1a50f.png)


You can further customize the appearance and behavior of your graph by modifying the settings or adding more nodes and edges as needed. Don't forget to apply changes returned from the widget.

### Interactive

You can check more advanced [interactive example](https://github.com/blitzarx1/egui_graph/tree/master/examples/interactive) for usage references, settings description and changes apply demonstration.
