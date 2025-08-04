# bevy_midi_params

[![Crates.io](https://img.shields.io/crates/v/bevy_midi_params.svg)](https://crates.io/crates/bevy_midi_params)
[![Docs.rs](https://docs.rs/bevy_midi_params/badge.svg)](https://docs.rs/bevy_midi_params)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yourusername/bevy_midi_params#license)

**Hardware MIDI controller integration for live parameter tweaking in Bevy games.**

Turn knobs, see results instantly. Perfect for shader artists, gameplay programmers, and technical artists who want tactile, real-time control over their game parameters.

## ✨ Features

- **🎛️ Zero-boilerplate MIDI mapping** - Just add `#[derive(MidiParams)]` and `#[midi(cc, range)]` attributes
- **💾 Live persistence** - Parameters auto-save as you tweak them, resume exactly where you left off
- **🔥 Hot reloading** - Change values with hardware controllers while your game runs
- **🎨 Auto-generated UI** - Inspect and fine-tune with automatically created egui interfaces
- **⚡ Plug & play** - Works with any MIDI controller (optimized for AKAI MIDImix)
- **🔗 Type-safe** - Full compile-time validation of CC mappings and ranges

## 🎮 Perfect For

- **Shader development** - Tweak roughness, metallic, emission in real-time
- **Physics tuning** - Adjust gravity, damping, forces with physical controls  
- **Procedural generation** - Modify noise parameters, generation rules on the fly
- **Audio/visual effects** - Control post-processing, particle systems, lighting
- **Game balancing** - Rapidly iterate on gameplay parameters

## 🚀 Quick Start

```toml
[dependencies]
bevy = "0.16"
bevy_midi_params = "0.1"
```

```rust
use bevy::prelude::*;
use bevy_midi_params::prelude::*;

#[derive(Resource, MidiParams)]
struct GameSettings {
    #[midi(1, 0.0..1.0)]          // Knob 1: Player speed
    pub player_speed: f32,
    
    #[midi(2, 0.0..20.0)]         // Knob 2: Gravity strength
    pub gravity: f32,
    
    #[midi(3, -1.0..1.0)]         // Knob 3: Camera shake
    pub camera_shake: f32,
    
    #[midi(33, button)]           // Button 1: Enable debug mode
    pub debug_mode: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            player_speed: 5.0,
            gravity: 9.81,
            camera_shake: 0.0,
            debug_mode: false,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MidiParamsPlugin::default())
        .run();
}
```

**That's it!** Your parameters now:
- ✅ Respond to MIDI controller input
- ✅ Auto-save when changed  
- ✅ Load previous values on startup
- ✅ Show up in a debug UI
- ✅ Work with both hardware and UI controls

## 🎛️ Supported Controllers

Tested with:
- **AKAI MIDImix** (recommended) - Perfect layout with 24 knobs + 8 faders + 16 buttons
- **Behringer BCR2000** - Tons of knobs for complex parameter sets
- **Novation Launch Control XL** - Great mix of knobs, faders, and pads
- Any MIDI controller with CC (Continuous Controller) support

## 📖 Examples

### Shader Tweaking
```rust
#[derive(Resource, MidiParams)]
struct MaterialParams {
    #[midi(1, 0.0..1.0)]    // Roughness
    pub roughness: f32,
    
    #[midi(2, 0.0..1.0)]    // Metallic  
    pub metallic: f32,
    
    #[midi(3, 0.0..3.0)]    // Emission intensity
    pub emission: f32,
}
```

### Physics Playground
```rust
#[derive(Resource, MidiParams)]
struct PhysicsSettings {
    #[midi(5, 0.0..20.0)]   // Gravity strength
    pub gravity: f32,
    
    #[midi(6, 0.0..1.0)]    // Air resistance
    pub damping: f32,
    
    #[midi(7, 0.1..5.0)]    // Time scale
    pub time_scale: f32,
    
    #[midi(34, button)]     // Pause simulation
    pub paused: bool,
}
```

### Advanced: Custom Ranges and UI
```rust
#[derive(Resource, MidiParams)]
struct AdvancedControls {
    #[midi(10, 0.1..100.0)]     // Exponential range
    pub particle_count: f32,
    
    #[midi(11, -180.0..180.0)]  // Degrees
    pub rotation_angle: f32,
    
    #[midi(12, 0.01..0.99)]     // Probability
    pub spawn_chance: f32,
}
```

## 🎵 MIDI CC Reference

For AKAI MIDImix:
- **Knobs 1-8**: CCs 1-8 (top row)
- **Knobs 9-16**: CCs 9-16 (middle row)  
- **Knobs 17-24**: CCs 17-24 (bottom row)
- **Faders 1-8**: CCs 19-26
- **Buttons**: CCs 33-40 (top), 49-56 (bottom)

## 🔧 Configuration

### Custom Persistence File
```rust
App::new()
    .add_plugins(MidiParamsPlugin::new("my_game_settings.ron"))
    .run();
```

### Disable UI (headless/release builds)
```toml
[dependencies]
bevy_midi_params = { version = "0.1", default-features = false }
```

## 🎯 Workflow

1. **Connect** your MIDI controller
2. **Derive** `MidiParams` on your structs  
3. **Annotate** fields with `#[midi(cc, range)]`
4. **Run** your game
5. **Tweak** parameters with physical knobs/faders
6. **Values auto-save** - restart and resume exactly where you left off!

## 🛠️ How It Works

bevy_midi_params uses:
- **Proc macros** to generate MIDI mapping and UI code
- **Inventory** for automatic registration of MIDI types
- **RON/JSON** for human-readable parameter persistence  
- **midir** for cross-platform MIDI input
- **egui** for optional debug UI (via bevy_egui)

No reflection needed, no manual registration, no boilerplate!

## 🤝 Compatible With

- ✅ `bevy-inspector-egui` - Use both for maximum flexibility
- ✅ Hot reloading workflows
- ✅ Asset pipeline integration
- ✅ Any Bevy version 0.16+

## 📋 Bevy Version Compatibility

| bevy_midi_params | Bevy |
|------------------|------|
| 0.1              | 0.16 |

## 🤝 Contributing

Contributions welcome! This crate is designed to be:
- **Beginner-friendly** - Clear error messages, good documentation
- **Extensible** - Easy to add new controller types and features
- **Fast** - Minimal runtime overhead

### Ideas for contributions:
- Additional controller presets
- Web MIDI support
- Parameter interpolation/smoothing
- MIDI learn functionality
- Integration with popular audio plugins

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

*Made with ❤️ for the Bevy community. Perfect your parameters, ship faster games.*
