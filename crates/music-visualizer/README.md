# Music Visualizer 🎵

A dynamic, audio-reactive fractal visualizer built with Rust, egui, and WebAssembly.

## Features

- **Fractal Tree Visualization**: Dynamic branching fractal that responds to audio
- **Audio Reactivity**: Parameters react to different frequency bands
  - Bass → Zoom & Width
  - Treble → Brightness
  - Spectral complexity → Depth
  - Beats → Rotation & Particle effects
- **Spectrum Analyzer**: Real-time frequency visualization
- **Waveform Display**: Live audio waveform
- **Beat Detection**: Visual pulses and particle effects on detected beats
- **Color Cycling**: Smooth color transitions through the spectrum
- **Demo Mode**: Built-in audio simulation for testing without microphone

## Audio Reactivity Mapping

| Audio Feature | Visual Parameter |
|--------------|------------------|
| Bass (20-250 Hz) | Zoom, Width, Pulse |
| Mids (500-2000 Hz) | Branch angles |
| Treble (4000+ Hz) | Brightness, Detail |
| Beats | Rotation kick, Particles |
| Spectral Centroid | Color hue shift |
| Volume | Glow intensity |

## Building

### Prerequisites

- Rust (latest stable)
- trunk: `cargo install trunk`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

### Development

```bash
# Start dev server with hot reload
trunk serve --port 8087 --open

# Or use the script
./serve.sh
```

### Production Build

```bash
# Build optimized release
trunk build --release

# Or use the script
./build.sh
```

Output will be in the `dist/` directory.

## Usage

1. **Demo Mode** (default): The visualizer runs with simulated audio
2. **Microphone Mode**: Click "Microphone" in the sidebar to use live audio input
   - Browser will request microphone permission
   - Works best with music playing nearby

## Configuration

All settings are adjustable in the left sidebar:

- **Fractal Settings**: Base zoom, width, depth, brightness
- **Audio Reactivity**: Multipliers for each audio→visual mapping
- **Animation**: Auto-rotate, rotation speed, beat pulse, color cycling
- **Display**: Toggle spectrum analyzer and waveform

## Technical Details

- Built with [egui](https://github.com/emilk/egui) for immediate-mode GUI
- Uses Web Audio API for real-time audio analysis
- FFT-based frequency band extraction
- Label propagation for beat detection
- Recursive fractal rendering with audio-modulated parameters

## License

MIT
