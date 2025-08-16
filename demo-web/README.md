# demo-web

Web (WASM) demo for `egui_graphs` using `eframe` + `wasm-bindgen`, built with `trunk`.

- Renders the `demo-core` app inside a `<canvas>` element (`#the_canvas_id`).
- Uses an import map to provide a tiny `env.now()` shim for timing.

## Prerequisites

- Rust toolchain with the wasm target:
  - `rustup target add wasm32-unknown-unknown`
- Trunk (Rust/Web bundler):
  - `cargo install trunk`

## Run locally

From the repository root or this folder:

```sh
cd demo-web
trunk serve --open
```

Trunk will build the WASM, start a dev server and open the demo in your browser.

If you prefer a one-off build:

```sh
cd demo-web
trunk build
# Output in `dist/` (served by Trunk)
```

## Project structure

- `index.html` — minimal page with a full-viewport `<canvas>` and `<link data-trunk rel="rust">` to build `demo-web`.
- `env.js` — small shim for `env.now()` used by some crates in WASM.
- `src/lib.rs` — bootstraps `eframe::WebRunner` and runs `demo_core::DemoApp`.
- `Cargo.toml` — sets `cdylib` and WASM-specific deps; uses `demo-core` and `egui_graphs` from the workspace.

## Troubleshooting

- If you see a blank page, open the browser devtools console — errors usually indicate a missing canvas id (`the_canvas_id`) or WASM initialization issue.
- Ensure the wasm target is installed and that you are on a recent Rust stable.
- If Trunk complains about permissions or an existing server, stop prior `trunk serve` instances.

## Features / Controls

Same as the native demo from `demo-core` — sliders, toggles, layout controls, and keybindings. See `demo-core/README.md` for details.
