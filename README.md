[![Crates.io](https://img.shields.io/crates/v/egui_graphs)](https://crates.io/crates/egui_graphs)
[![docs.rs](https://img.shields.io/docsrs/egui_graphs)](https://docs.rs/egui_graphs)

# egui_graphs
Grpah visualization implementation using [egui](https://github.com/emilk/egui)

<img width="798" alt="Screenshot 2023-04-21 at 19 16 34" src="https://user-images.githubusercontent.com/32969427/233673151-0072378d-25a4-4066-bbff-2042ddf6b3fe.png">
<img width="798" src="https://user-images.githubusercontent.com/32969427/233801083-160cb9dd-e2a5-4292-8c59-f2143f31588c.png">

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
------------------------------+-------
basic graph drawing                  | [x]
self-references, multi-connections   | [x]
zoom & pan                           | [x]
drag node                            | [x]
select/deselect node                 | [x]
select/multi-select                  | [ ]
style customizations                 | [ ]
support egui dark/light theme        | [ ]
interactions vs egui draw benchmarks | [ ]
documentation, tests, example        | [ ]
</pre>

## Example
You can also check the [example](https://github.com/blitzarx1/egui_graph/tree/master/example) for usage references and settings description.
