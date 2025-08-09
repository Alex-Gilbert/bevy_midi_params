//! Example showing how bevy_midi_params and bevy_persist work together
//! 
//! This demonstrates the clean separation:
//! - MIDI control for development/tweaking
//! - Persistence that works in both dev and production

use bevy::prelude::*;
use bevy_midi_params::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};

// Define your game settings with both MIDI control and persistence
#[derive(Resource, Default, MidiParams, Serialize, Deserialize)]
struct GameSettings {
    #[midi(1, 0.0..10.0)]
    pub player_speed: f32,
    
    #[midi(2, 0.0..20.0)]
    pub jump_height: f32,
    
    #[midi(3, -1.0..1.0)]
    pub camera_shake: f32,
    
    #[midi(16, button)]
    pub debug_mode: bool,
    
    // Non-MIDI fields are still persisted
    pub player_name: String,
    pub high_score: u32,
}

// Implement Persistable for automatic saving
impl Persistable for GameSettings {
    fn type_name() -> &'static str {
        "GameSettings"
    }
}

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Add persistence (always active)
    app.add_plugins(PersistPlugin::new("game_settings.ron"));
    
    // Add MIDI control (only in dev builds)
    #[cfg(debug_assertions)]
    app.add_plugins(MidiParamsPlugin::new());
    
    // Register the settings resource
    app.init_resource::<GameSettings>();
    
    // Register for auto-persistence
    register_persistable!(app, GameSettings);
    
    // Listen for MIDI changes to provide feedback
    #[cfg(debug_assertions)]
    app.add_systems(Update, on_midi_change);
    
    app.add_systems(Startup, setup);
    app.add_systems(Update, game_logic);
    
    app.run();
}

fn setup(mut commands: Commands, settings: Res<GameSettings>) {
    commands.spawn(Camera2d);
    
    println!("Game Settings Loaded:");
    println!("  Player Speed: {}", settings.player_speed);
    println!("  Jump Height: {}", settings.jump_height);
    println!("  Camera Shake: {}", settings.camera_shake);
    println!("  Debug Mode: {}", settings.debug_mode);
    println!("  Player Name: {}", settings.player_name);
    println!("  High Score: {}", settings.high_score);
    
    #[cfg(debug_assertions)]
    println!("\n[DEV MODE] Connect a MIDI controller to adjust parameters live!");
    
    println!("\nPress SPACE to test persistence (changes will auto-save)");
}

#[cfg(debug_assertions)]
fn on_midi_change(mut events: EventReader<MidiParamChanged>) {
    for event in events.read() {
        println!("MIDI Changed: {} CC{} = {:.2}", 
            event.type_name, event.cc, event.value);
    }
}

fn game_logic(
    mut settings: ResMut<GameSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Simulate gameplay that uses the settings
    if keyboard.just_pressed(KeyCode::Space) {
        // Modify non-MIDI fields to test persistence
        settings.high_score += 100;
        println!("High score increased to: {}", settings.high_score);
    }
    
    // Use the MIDI-controlled parameters in your game
    // These values can be tweaked live via MIDI in dev builds
    // and will persist between runs
    let player_velocity = settings.player_speed * time.delta_secs();
    // ... use in actual game logic
}