# Animated nodes
Example demonstrates how to use custom drawing functions nodes, handle animation state and access `node` payload in the drawing function. 

Nodes are drawn as squares with labels in their center. Node is rotating when it is in the state of dragging. It is rotating clockwise if the `node` payload has `clockwise` field setting set to `true` and vice-verse.

## run
```bash
cargo run --release -p animated_nodes
```
