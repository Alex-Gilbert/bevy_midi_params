use bevy::prelude::*;
use bevy_midi_params::prelude::*;

/// Example demonstrating the new two-plugin architecture
/// 
/// This example shows how to use:
/// - Parameters with MIDI control (CC numbers)
/// - Parameters that are persist-only (no MIDI control)
/// - Seamless dev-to-production switching
#[derive(Resource, MidiParams, Default)]
struct GameSettings {
    // MIDI-controlled parameters (available in dev builds with "midi" feature)
    #[midi(16, 0.0..10.0)]
    pub player_speed: f32,
    
    #[midi(2, button)]
    pub debug_mode: bool,
    
    #[midi(3, 0.0..100.0)]
    pub enemy_health: f32,
    
    // Persist-only parameters (always saved, never MIDI controlled)
    #[midi(persist, 0.0..1.0)]
    pub master_volume: f32,
    
    #[midi(persist, button)]
    pub fullscreen: bool,
    
    #[midi(persist, 0..=10)]
    pub graphics_quality: i32,
}

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // For development builds (with MIDI support):
    #[cfg(feature = "dev")]
    app.add_plugins(bevy_midi_params::dev_plugins());
    
    // For production builds (persistence only):
    #[cfg(not(feature = "dev"))]
    app.add_plugins(bevy_midi_params::prod_plugins());
    
    // Alternative: Add plugins individually for more control
    // app.add_plugins(ParamsPersistencePlugin::default().with_persist("game_settings.ron"));
    // #[cfg(feature = "midi")]
    // app.add_plugins(MidiControlPlugin::default().with_controller("MIDI Fighter"));
    
    app.add_systems(Update, (
        demo_system,
        #[cfg(feature = "ui")]
        ui_system,
    ));
    
    app.run();
}

fn demo_system(settings: Res<GameSettings>) {
    // Use the settings in your game logic
    // These values will be:
    // - Loaded from file on startup
    // - Updated by MIDI in dev builds (if feature enabled)
    // - Saved automatically when changed
    
    if settings.is_changed() {
        info!("Settings changed:");
        info!("  Player speed: {}", settings.player_speed);
        info!("  Debug mode: {}", settings.debug_mode);
        info!("  Enemy health: {}", settings.enemy_health);
        info!("  Master volume: {}", settings.master_volume);
        info!("  Fullscreen: {}", settings.fullscreen);
        info!("  Graphics quality: {}", settings.graphics_quality);
    }
}

#[cfg(feature = "ui")]
fn ui_system(
    mut contexts: bevy_egui::EguiContexts,
    mut settings: ResMut<GameSettings>,
) {
    bevy_egui::egui::Window::new("Game Settings")
        .show(contexts.ctx_mut(), |ui| {
            settings.render_ui(ui);
        });
}

// Example showing how the same code works in both dev and production:
// 
// Development build (Cargo.toml features = ["dev"]):
// - MIDI controls work for CC 1, 2, 3
// - All parameters are persisted to file
// - UI shows "CC1", "CC2", "CC3" labels and "persist only" labels
// 
// Production build (Cargo.toml features = ["persistence"]):
// - No MIDI dependencies compiled in
// - All parameters still persisted to file
// - UI shows "persist only" labels for all parameters
// - Zero runtime overhead from MIDI code
