# Changelog

## v0.28.0 (23.08.2025)

### 🚀Highlights

#### 🆕 New Features

- We are web now
- File import and export in demo (will be added to core in the next releases)
- Changelog added

#### 🖥️  Demo

- File import (in web demo for now... plan to move it to `egui_graphs` next release)
  - User uploads
    - JSON schema description
      - only edges
      - nodes and edges
      - graph and layout properties
  - Beautiful example graphs
- File export
  - Optional include of layout and graph settings

#### 🛠️ Fixes & Robustness

### MRs TODO:

**Full increment**: TODO:

## v0.27.0 (16.08.2025)

### 🚀Highlights

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

### Highlights

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