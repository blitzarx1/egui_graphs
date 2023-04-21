[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs
Grpah visualization implementation using [egui](https://github.com/emilk/egui)

<img width="798" alt="Screenshot 2023-04-21 at 19 16 34" src="https://user-images.githubusercontent.com/32969427/233673151-0072378d-25a4-4066-bbff-2042ddf6b3fe.png">

## Status
The project is close to the first stable version.

Currently not optimized for large graphs. The goal is to match egui drawing speed. Further optimizations are unnecessary.

## Concept
The goal is to create a crate that expands egui's visualization capabilities and offers an easy-to-integrate, customizable graph visualization widget.

* Customization and interactivity;
* Ability to draw arbitrarily complex graphs with self-references, loops, etc.;
* Widget does not modify the provided graph and properties; instead, it generates changes in case of any interactions which the client can apply.

## Features
<pre>
feature                status
----------------------+-------
force directed layout | [x]
zoom                  | [x]
pan                   | [x]
drag                  | [x]
simulation settings   | [x]
select/deselect node  | [ ]
add/delete node       | [ ]
fold/unfold node      | [ ]
layout customizations | [ ]
style customizations  | [ ]
</pre>

## Example
You can also check the [example](https://github.com/blitzarx1/egui_graph/tree/master/example) for usage references and settings description.
