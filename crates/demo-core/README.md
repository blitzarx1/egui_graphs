# demo-core

Shared demo app for `egui_graphs`, built with `egui` + `eframe`. This crate is consumed by:

- Native example: `cargo run --release --example demo`
- Web demo (WASM): `demo-web`

## Run the native demo

From the repository root:

```sh
cargo run --release --example demo
```

With interaction events UI enabled (optional feature):

```sh
cargo run --release --example demo --features events
```

## What’s included

- Random graph generator with adjustable node/edge counts
- Force-directed layout (Fruchterman–Reingold + optional center gravity), with live tuning
- Interaction settings (dragging, hover, clicking, selection, multi-selection)
- Navigation (fit-to-screen, zoom & pan, zoom speed)
- Style toggles (labels always on, edge deemphasis)
- Lightweight debug overlay (FPS, counts, steps)

## Keybindings (quick reference)

- Nodes: `n` (+1), `Shift+n` (−1), `m` (+10 up to max), `Shift+m` (−10)
- Edges: `e` (+1), `Shift+e` (−1), `r` (+10 up to max), `Shift+r` (−10)
- Swap: `Ctrl+Shift+n|m|e|r` (remove then add)
- UI: `d` (toggle debug overlay), `Tab` (toggle side panel), `Space` (reset), `h` or `?` (keybindings)
- Navigation: drag to pan, `Ctrl+wheel` to zoom (when enabled)

Notes

- Max limits: nodes = 2500, edges = 5000
- The “events” feature gates event publishing/filters and extra overlay values

## Using `DemoApp` in your own app

```rust
eframe::run_native(
    "egui_graphs demo",
    eframe::NativeOptions::default(),
    Box::new(|cc| Ok::<Box<dyn eframe::App>, _>(Box::new(demo_core::DemoApp::new(cc))))
)?;
```
