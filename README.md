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
select/deselect                      | [x]
select/multi-select                  | [x]
style customizations                 | [ ]
support egui dark/light theme        | [ ]
interactions vs egui draw benchmarks | [ ]
documentation, tests, example        | [ ]
</pre>

## Example

![ezgif-4-3e4e4469e6](https://user-images.githubusercontent.com/32969427/233863786-11459176-b741-4343-8b42-7d9b3a8239ee.gif)

You can also check the [example](https://github.com/blitzarx1/egui_graph/tree/master/example) for usage references and settings description.
