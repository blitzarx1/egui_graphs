# Code Analyzer Web Demo

A comprehensive web-based demonstration of `egui_graphs` capabilities, featuring interactive graph visualization, network stress testing, and neural network simulation.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Framework](https://img.shields.io/badge/framework-eframe%200.33-orange)
![WASM](https://img.shields.io/badge/wasm-supported-green)

## 🌟 Features

### 📊 Graph Visualization
- **2D/3D rendering modes** with smooth transitions
- **Interactive nodes and edges** with hover effects
- **Force-directed layout** with real-time simulation
- **Customizable styling** for nodes, edges, and background
- **Zoom and pan** navigation with smooth controls
- **Center graph** button to fit viewport

### 🚨 DDoS Stress Test Simulator
Real-time network attack simulation with comprehensive metrics:

#### Attack Patterns
- **Flood Attack**: High volume request spam
- **Slowloris**: Slow connection exhaustion
- **SYN Flood**: TCP handshake overflow
- **UDP Flood**: UDP packet bombardment
- **HTTP Flood**: Application layer saturation

#### Metrics Dashboard
- Total requests (successful/failed)
- Read/write operations counters
- Real-time throughput monitoring
- Average response time
- Peak throughput tracking
- Elapsed time

#### Visualization
- **Custom throughput graph** with gradient fill
- **Color-coded logging system**:
  - 🔵 Info (blue)
  - 🟡 Warning (yellow)
  - 🔴 Error (red)
  - 🔴 Critical (bright red)
- Timestamped log entries with 1000-entry buffer

### 🧠 Neural Network Simulator
Interactive neural network visualization with live signal propagation:

#### Configurable Architecture
- **Input layers**: 1-3 layers
- **Hidden layers**: 1-5 layers  
- **Output layers**: 1-3 layers
- **Neurons per layer**: Customizable for each layer type

#### Visualization Features
- **Real-time neuron firing** with color changes
- **Signal propagation** through weighted synapses
- **Activation levels** shown as inner circles
- **Automatic layout** arranged in traditional NN diagram style
- **Layer-based organization** with proper spacing

#### Customization
- **Neuron colors**:
  - Inactive neurons (default: blue-gray)
  - Firing neurons (default: yellow-orange)
  - Input neurons (default: green)
  - Output neurons (default: purple)
- **Synapse colors**:
  - Inactive synapses (default: dark gray)
  - Active synapses (default: bright orange)
- **Simulation parameters**:
  - Fire rate: 0.1-10 Hz
  - Propagation speed: 0.1-2.0x

#### Statistics
- Real-time firing neuron count
- Total neuron count
- Network architecture summary

### 💾 Import/Export System
- **JSON format**: Complete graph structure with metadata
- **CSV format**: Node and edge listings
- **Graphviz DOT format**: For external visualization tools
- **Browser download** via Blob API

### 🎨 Customization Panel
Comprehensive settings for all visual aspects:
- Node colors (default, hover, selected)
- Edge colors (default, selected)
- Background color
- 3D rotation controls (X, Y, Z axes)
- Auto-rotation with speed control
- Depth effects (fade, scale)
- Sphere rendering mode

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk
```

### Build and Run

#### Development mode
```bash
cd crates/code-analyzer-web
trunk serve
```
Opens at http://127.0.0.1:8080

#### Release mode (optimized)
```bash
cd crates/code-analyzer-web
trunk serve --release
```

#### Build static assets
```bash
trunk build --release
# Output in dist/ directory
```

## 📁 Project Structure

```
code-analyzer-web/
├── src/
│   └── lib.rs          # Main application logic
├── assets/             # JSON graph examples
│   ├── bipartite.json
│   ├── cliques.json
│   ├── grid.json
│   └── ...
├── index.html          # HTML template
├── Cargo.toml          # Dependencies
└── build.sh            # Build script for WSL
```

## 🎯 Usage Guide

### Graph Tab
1. Select visualization mode (2D/3D)
2. Click nodes to select them
3. View class details in the sidebar
4. Use "Center Graph" to fit viewport
5. Adjust settings via configuration panel

### Stress Test Tab
1. Select attack pattern from dropdown
2. Adjust attack intensity (0-100%)
3. Set max requests per second
4. Click "Start Attack" to begin simulation
5. Monitor real-time metrics and throughput graph
6. View detailed logs in the Logs tab

### Neural Network Tab
1. Configure network architecture:
   - Number of input/hidden/output layers
   - Neurons per layer for each type
2. Customize colors for each neuron type
3. Adjust simulation parameters (fire rate, speed)
4. Click "Generate Neural Network"
5. Watch neurons fire and signals propagate
6. Use "Center Graph" for better viewing

### Configuration
- Access via "⚙️ Settings" button or menu
- Search settings by keyword
- Toggle auto-save to persist preferences
- Reset to defaults anytime

## 🛠️ Technical Stack

### Core Technologies
- **eframe 0.33**: Application framework
- **egui 0.33**: Immediate mode GUI
- **egui_graphs 0.29**: Graph visualization widget
- **petgraph 0.8**: Graph data structures
- **web-sys**: Browser API bindings
- **js-sys**: JavaScript interop

### Architecture
- **WASM compilation** for web deployment
- **Modular design** with tab-based UI
- **State management** with persistent configuration
- **Real-time simulation** with frame-based updates
- **Custom rendering** for specialized visualizations

## 📊 Performance

### Optimization
- Release builds use `opt-level = "z"` for size
- LTO enabled for smaller WASM binaries
- Efficient graph algorithms (O(n²) force-directed)
- Throttled updates for smooth animation

### Recommended Specs
- Modern browser with WebGL support
- 4GB+ RAM for large graphs (1000+ nodes)
- For neural networks: Best with < 100 total neurons

## 🐛 Troubleshooting

### Build Issues
```bash
# Clean build
cargo clean
trunk clean

# Rebuild
trunk serve --release
```

### WSL Issues
If encountering linker errors on WSL:
```bash
# Ensure build-essential is installed
sudo apt-get install build-essential

# Create cargo config
mkdir -p ~/.cargo
echo '[target.x86_64-unknown-linux-gnu]' > ~/.cargo/config.toml
echo 'linker = "/usr/bin/gcc"' >> ~/.cargo/config.toml
```

### Port Already in Use
```bash
# Kill existing trunk process
pkill trunk

# Or specify different port
trunk serve --port 8081
```

## 📝 License

This demo application is part of the egui_graphs project. See the main repository LICENSE file for details.

## 🤝 Contributing

Contributions welcome! This demo showcases advanced features of egui_graphs. Feel free to:
- Add new visualization modes
- Implement additional graph algorithms
- Enhance UI/UX
- Optimize performance
- Add more export formats

## 🔗 Links

- [egui_graphs repository](https://github.com/blitzar-tech/egui_graphs)
- [egui documentation](https://docs.rs/egui)
- [petgraph documentation](https://docs.rs/petgraph)
- [Trunk documentation](https://trunkrs.dev)

## 📧 Support

For issues specific to this demo application, please check:
1. Main repository issues
2. egui_graphs documentation
3. Rust WASM compatibility

---

**Built with ❤️ using Rust, egui, and modern web technologies**
