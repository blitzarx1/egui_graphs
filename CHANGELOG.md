# Changelog

## v0.29.0 (31.10.2025)

### MRs
* Feature: custom egui ids by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/292
* Fix: enable instant/wasm-bindgen at workspace level and remove from demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/293
* Update egui to 0.33 (and update bevy, ureq, and criterion) by @oscargus in https://github.com/blitzar-tech/egui_graphs/pull/298
* Fix: replace 'instant' dependency with 'web-time' by @WaffleSoul4 in https://github.com/blitzar-tech/egui_graphs/pull/296

### New Contributors
* @oscargus made their first contribution in https://github.com/blitzar-tech/egui_graphs/pull/298
* @WaffleSoul4 made their first contribution in https://github.com/blitzar-tech/egui_graphs/pull/296

**Full Increment**: https://github.com/blitzar-tech/egui_graphs/compare/v0.28.0...v0.29.0

## v0.28.0 (23.08.2025)

#### 🆕 New Features

A lot of work done on Demo. The most important is that we are web now!

— How to try it:

1. Open the web demo and pick an example:
   - [bipartite.json](https://blitzar-tech.github.io/egui_graphs/#g=bipartite.json)
   - [tree_binary.json](https://blitzar-tech.github.io/egui_graphs/#g=tree_binary.json)
2. Tweak Layout → Force-Directed/Hierarchical and play with Animation/Forces.
3. Click Export (top of the right panel) → choose “Include Layout” and/or “Node Positions”, then save to File or copy to Clipboard.
4. To import, go to the Import/Load tab → “Open” (or drag & drop a JSON file into the graph area). Your uploads appear under “User Uploads”.
5. On web, use Share to copy a deep link to the selected example.

- More useful keybindings were added (fit to screen (once), pan to graph)
- File import and export — planned to migrate to `egui_graphs` core in upcoming releases

Notes:

- New import/export features are in the demo UI only (not yet in the `egui_graphs` crate). Core support is planned.
- Positions are optional on import; if missing, nodes are placed in a circle initially.

#### 🖥️  Demo

- File import:
  - User uploads with JSON schema support
    - Edges-only
    - Nodes and edges
    - Graph plus layout properties
  - Curated example graphs
  - Shareable deep links (see Highlights above)
- File export:
  - Optional include of layout and graph settings
- Demo UX improvements:
  - Debug overlay and instructional messages queue
  - Navigation keybindings and help button
  - Hierarchical layout controls and perf metrics panel

#### 🛠️ Fixes & Robustness

- Do not send zero-diff events (core)

### MRs:

- Feature: web demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/272
- Chore: update web demo link by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/273
- Fix: enable events for web by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/274
- Feature: demo debug overlay improvements by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/275
- Feature: instructional messages queue for demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/276
- Feature: help button in demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/277
- Feature: hierarchical layout demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/278
- Fix: redraw hierarchical layout on reset by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/279
- Fix: do not send zero diff events by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/280
- Feature: perf metrics for demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/281
- Feature: info messages updates by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/282
- Chore: refactor demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/283
- Feature: file import for demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/284
- Feature: navigation keybindings and notifications by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/285
- Feature: url params and share in demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/286

**Full increment**: https://github.com/blitzar-tech/egui_graphs/compare/v0.27.0...v0.28.0

## v0.27.0 (16.08.2025)

#### 🆕 New Features

- Hover Interactions: Added from scratch! Graph nodes and edges now support hover effects, enabling more interactive and intuitive graph exploration.
- Custom Styling Hooks: Support for node and edge style hooks, allowing flexible and dynamic visual customization.
- Force-Directed Layout Enhancements:
  - Fruchterman-Reingold algorithm and extra forces for any force-directed graph.
  - Fast-forward feature for animated layouts.
  - Exposed force with an Extras wrapper for advanced usage.

#### 🖥️  Demo

- Event filters, show/hide panels, and enhanced keybindings for better demo interactivity.
- Keybindings overlay replaced with a modern modal window.
- Debug overlay now displays steps count for animated layouts.
- Synchronized sliders and keybindings for a smoother demo experience.

#### 🛠️ Fixes & Robustness

- Fixed 1-frame edge glitch and improved edge overlap handling.
- Fit-to-screen now works for single-node graphs.
- Guards for empty graphs and demo refactoring for robustness.
- Prevented negative tolerance in bezier curves.
- Ensured the graph is fully serde serializable/deserializable.

### MRs

- Support for node and edges style hooks by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/250
- Update README.md by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/251
- Fix: 1 frame edge glitch by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/252
- Fix/fit to screen 1 node by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/253
- Guards for empty graph and refactor demo example by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/255
- Events filter in demo example by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/256
- Demo show/hide panel and keybindings by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/257
- Fix sync sliders and keybindings by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/258
- Replace keybindings overlay with modal window by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/261
- Feature: fruchterman_reingold && extra forces for any fdg by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/263
- Feature: hover by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/264
- Cleanup: Expose force with Extras wrapper and modify README by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/265
- Feature: fast-forward for animated layouts by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/266
- Fix: avoid negative tolerance for bezier curves by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/267
- Feature: steps count for debug overlay in demo by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/268
- Fix: overlapping edges of order 1 by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/269
- Fix: ensure graph is serde serializable/deserializable by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/271

**Full increment**: https://github.com/blitzar-tech/egui_graphs/compare/v0.26.0...v0.27.0

## v0.26.0 (09.08.2025)

#### 🆕 New Features

- Added naive force-directed layout (Fruchterman–Reingold style) with adjustable simulation parameters.
- Added layout state get/set API on `GraphView` for external control/persistence.

#### 🖥️  Demo

- Larger debug overlay text in demo.
- Demo now has a Force Directed panel (sliders + info tooltips) for live tuning.

#### 🛠️ Fixes & Robustness

- Bumped `egui` to `0.32` (and refreshed related dev dependencies).
- Refactored layout logic into smaller private helpers and added physics unit tests.

### MRs

- Renaming and API stabilization by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/240
- Add Layout and LayoutState to public scope by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/248
- FDG layout, egui bump by @blitzarx1 in https://github.com/blitzar-tech/egui_graphs/pull/249

**Full increment**: https://github.com/blitzar-tech/egui_graphs/compare/v0.25.1...v0.26.0
