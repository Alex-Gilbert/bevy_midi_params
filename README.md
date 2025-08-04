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
- **⚡ Plug & play** - Works with any MIDI controller with CC support
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

## 🎛️ Finding Your Controller's CC Values

To use any MIDI controller, you need to find which CC (Continuous Controller) numbers your knobs, faders, and buttons send. Use `aseqdump` to discover these values:

### Install aseqdump (Linux/macOS)
```bash
# Ubuntu/Debian
sudo apt install alsa-utils

# macOS (via Homebrew)
brew install alsa-utils
```

### Find Your Controller's CC Numbers
1. **Connect your MIDI controller**
2. **Run aseqdump to list MIDI ports:**
   ```bash
   aseqdump -l
   ```
3. **Start monitoring your controller** (replace `20:0` with your controller's port):
   ```bash
   aseqdump -p 20:0
   ```
4. **Turn knobs/move faders** - you'll see output like:
   ```
   Waiting for data. Press Ctrl+C to end.
   Source_ 20:0, Event type = 10 (Control change), Channel = 0, Control = 1, Value = 64
   Source_ 20:0, Event type = 10 (Control change), Channel = 0, Control = 2, Value = 127
   ```
5. **Note the Control numbers** - these are your CC values to use in `#[midi(CC, range)]`

## 🎵 Using Your CC Values

Once you've found your controller's CC numbers with `aseqdump`, use them in your code:

```rust
#[derive(Resource, MidiParams)]
struct MyControls {
    #[midi(1, 0.0..10.0)]    // CC 1 from aseqdump
    pub speed: f32,
    
    #[midi(7, 0.0..1.0)]     // CC 7 from aseqdump  
    pub volume: f32,
    
    #[midi(64, button)]      // CC 64 button from aseqdump
    pub enabled: bool,
}
```

## 🔧 Configuration

### Custom Configuration
```rust
// Default - uses first available controller
App::new()
    .add_plugins(MidiParamsPlugin::default())
    .run();

// Specify controller name (partial match)
App::new()
    .add_plugins(MidiParamsPlugin::new().with_controller("Launch Control"))
    .run();

// Custom persistence file
App::new()
    .add_plugins(MidiParamsPlugin::new().with_persist("my_settings.ron"))
    .run();

// Chain multiple options
App::new()
    .add_plugins(
        MidiParamsPlugin::new()
            .with_controller("BCR2000")
            .with_persist("bcr_settings.ron")
    )
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
- **RON** for human-readable parameter persistence  
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
