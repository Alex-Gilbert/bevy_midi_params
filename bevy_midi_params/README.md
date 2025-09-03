# bevy_midi_params

Hardware MIDI controller integration for live parameter tweaking in Bevy games.

## Features

- **MIDI Controller Support**: Connect any MIDI controller to adjust game parameters in real-time
- **Auto-discovery**: Automatically detects and connects to MIDI devices
- **Derive Macro**: Simple `#[derive(MidiParams)]` to expose any component's fields to MIDI control
- **Live Tweaking**: Perfect for game development, balancing, and creative experimentation
- **Flexible Mapping**: Map MIDI CC messages to any numeric parameter with custom ranges

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_midi_params = "0.1"
```

Expose component fields to MIDI control:

```rust
use bevy::prelude::*;
use bevy_midi_params::prelude::*;

#[derive(Component, MidiParams)]
struct Player {
    #[midi(cc = 1, range = 0.0..=2.0)]
    speed: f32,
    
    #[midi(cc = 2, range = 0.0..=10.0)]
    jump_force: f32,
    
    #[midi(cc = 3)]
    health: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MidiParamsPlugin)
        .run();
}
```

## Usage

1. Connect your MIDI controller to your computer
2. Run your Bevy application
3. The plugin will automatically detect and connect to available MIDI devices
4. Use your controller's knobs/sliders to adjust the mapped parameters in real-time

## Advanced Configuration

### Custom CC Mappings

```rust
#[derive(Component, MidiParams)]
struct GameConfig {
    #[midi(cc = 10, range = 0.1..=5.0)]
    time_scale: f32,
    
    #[midi(cc = 11, range = 0.0..=1.0)]
    fog_density: f32,
    
    #[midi(cc = 12, range = 1..=100)]
    enemy_count: u32,
}
```

### MIDI Device Selection

```rust
fn select_midi_device(mut midi: ResMut<MidiConnection>) {
    // List available devices
    for device in midi.available_devices() {
        println!("Device: {}", device.name);
    }
    
    // Connect to specific device
    midi.connect_to("Akai MPK Mini");
}
```

## Compatible Controllers

Works with any MIDI controller that sends CC (Control Change) messages, including:
- Akai MPK series
- Novation Launch Control
- Korg nanoKONTROL
- Behringer X-Touch
- And many more!

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.