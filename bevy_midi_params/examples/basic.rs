//! Basic example showing MIDI parameter control with Bevy.
//!
//! This example demonstrates:
//! - Setting up MIDI parameter control
//! - Using different control types (ranges and buttons)
//! - Auto-persistence of parameter values
//!
//! Controls (use `aseqdump` to find your controller's CC values):
//! - CC 16: Player speed (0.0 - 10.0)
//! - CC 20: Jump height (1.0 - 5.0)
//! - CC 33: Enable debug mode (button)
//! - CC 34: Enable physics (button)
//!
//! The cube will change color based on the player speed parameter.

use bevy::prelude::*;
use bevy_midi_params::prelude::*;

#[derive(Resource, MidiParams)]
struct GameSettings {
    #[midi(16, 0.0..10.0)]
    pub player_speed: f32,

    #[midi(20, 1.0..5.0)]
    pub jump_height: f32,

    #[midi(24, button)]
    pub debug_mode: bool,

    #[midi(28, button)]
    pub physics_enabled: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            player_speed: 5.0,
            jump_height: 2.0,
            debug_mode: false,
            physics_enabled: true,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            MidiParamsPlugin::new()
                .with_controller("midi mix")
                .with_persist("basic_example.ron"),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (update_cube_color, display_debug_info))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(2.0, 2.0, 2.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.2, 0.3),
        ..default()
    });
    let material = MeshMaterial3d(material);

    // Spawn a cube that will change based on parameters
    commands.spawn((
        Mesh3d(cube_mesh),
        material,
        Transform::from_xyz(0.0, 0.0, 0.0),
        DemoCube,
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

#[derive(Component)]
struct DemoCube;

fn update_cube_color(
    settings: Res<GameSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_query: Query<&MeshMaterial3d<StandardMaterial>, With<DemoCube>>,
) {
    if !settings.is_changed() {
        return;
    }

    for material_handle in cube_query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            // Color changes based on player speed
            let speed_factor = settings.player_speed / 10.0;
            material.base_color = Color::srgb(speed_factor, 1.0 - speed_factor, 0.5);

            // Emission based on jump height
            let jump_factor = (settings.jump_height - 1.0) / 4.0;
            material.emissive = Color::srgb(jump_factor * 0.5, 0.0, jump_factor * 0.5).into();
        }
    }
}

fn display_debug_info(settings: Res<GameSettings>, mut gizmos: Gizmos) {
    if settings.debug_mode {
        // Draw debug gizmos
        gizmos.cuboid(
            Transform::from_xyz(0.0, 0.0, 0.0),
            Color::srgb(0.0, 1.0, 0.0),
        );

        // Print values to console when they change
        if settings.is_changed() {
            println!("🎮 Game Settings Updated:");
            println!("  Player Speed: {:.2}", settings.player_speed);
            println!("  Jump Height: {:.2}", settings.jump_height);
            println!("  Debug Mode: {}", settings.debug_mode);
            println!(
                "  Physics: {}",
                if settings.physics_enabled {
                    "ON"
                } else {
                    "OFF"
                }
            );
        }
    }
}
