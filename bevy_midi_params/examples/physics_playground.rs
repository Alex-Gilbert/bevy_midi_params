//! Physics playground example showing real-time physics parameter tweaking.
//! 
//! Perfect for gameplay programmers who need to rapidly iterate on physics feel.
//! 
//! Controls (AKAI MIDImix):
//! - Knob 1: Gravity strength (0.0 - 30.0)
//! - Knob 2: Air resistance (0.0 - 0.5)
//! - Knob 3: Bounce damping (0.0 - 1.0)
//! - Knob 4: Time scale (0.1 - 3.0)
//! - Button 1: Pause physics
//! - Button 2: Reset scene

use bevy::prelude::*;
use bevy_midi_params::prelude::*;
use std::f32::consts::PI;

#[derive(Resource, MidiParams)]
struct PhysicsParams {
    #[midi(1, 0.0..30.0)]
    pub gravity_strength: f32,
    
    #[midi(2, 0.0..0.5)]
    pub air_resistance: f32,
    
    #[midi(3, 0.0..1.0)]
    pub bounce_damping: f32,
    
    #[midi(4, 0.1..3.0)]
    pub time_scale: f32,
    
    #[midi(33, button)]
    pub paused: bool,
    
    #[midi(34, button)]
    pub reset_scene: bool,
}

impl Default for PhysicsParams {
    fn default() -> Self {
        Self {
            gravity_strength: 9.81,
            air_resistance: 0.02,
            bounce_damping: 0.8,
            time_scale: 1.0,
            paused: false,
            reset_scene: false,
        }
    }
}

#[derive(Component)]
struct PhysicsObject {
    velocity: Vec3,
    mass: f32,
}

#[derive(Component)]
struct Bouncy;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MidiParamsPlugin::new("physics_playground.ron"))
        .add_systems(Startup, setup_physics_scene)
        .add_systems(Update, (
            physics_simulation,
            handle_reset,
            spawn_random_objects,
        ))
        .run();
}

fn setup_physics_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(20.0, 20.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.5, 0.3),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Spawn some initial physics objects
    spawn_physics_objects(&mut commands, &mut meshes, &mut materials, 5);

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 15.0).looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -0.5)),
        ..default()
    });
}

fn spawn_physics_objects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    count: usize,
) {
    use bevy::math::*;
    
    for i in 0..count {
        let x = (i as f32 - count as f32 / 2.0) * 2.0;
        let y = 10.0 + i as f32 * 2.0;
        
        let color = Color::hsl(i as f32 * 60.0, 0.8, 0.6);
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.5)),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                }),
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            PhysicsObject {
                velocity: Vec3::new(
                    (i as f32 - count as f32 / 2.0) * 0.5,
                    0.0,
                    0.0,
                ),
                mass: 1.0,
            },
            Bouncy,
        ));
    }
}

fn physics_simulation(
    time: Res<Time>,
    params: Res<PhysicsParams>,
    mut physics_query: Query<(&mut Transform, &mut PhysicsObject), With<Bouncy>>,
) {
    if params.paused {
        return;
    }

    let dt = time.delta_seconds() * params.time_scale;
    let gravity = Vec3::new(0.0, -params.gravity_strength, 0.0);
    
    for (mut transform, mut physics) in physics_query.iter_mut() {
        // Apply gravity
        physics.velocity += gravity * dt;
        
        // Apply air resistance
        physics.velocity *= 1.0 - params.air_resistance;
        
        // Update position
        transform.translation += physics.velocity * dt;
        
        // Ground collision and bouncing
        if transform.translation.y < 0.5 {
            transform.translation.y = 0.5;
            physics.velocity.y = -physics.velocity.y * params.bounce_damping;
        }
        
        // Wall collisions
        if transform.translation.x.abs() > 10.0 {
            transform.translation.x = transform.translation.x.signum() * 10.0;
            physics.velocity.x = -physics.velocity.x * params.bounce_damping;
        }
        
        if transform.translation.z.abs() > 10.0 {
            transform.translation.z = transform.translation.z.signum() * 10.0;
            physics.velocity.z = -physics.velocity.z * params.bounce_damping;
        }
    }
    
    // Print physics info when parameters change
    if params.is_changed() {
        println!("⚡ Physics Parameters:");
        println!("  Gravity: {:.2} m/s²", params.gravity_strength);
        println!("  Air Resistance: {:.3}", params.air_resistance);
        println!("  Bounce Damping: {:.3}", params.bounce_damping);
        println!("  Time Scale: {:.2}x", params.time_scale);
        println!("  Status: {}", if params.paused { "PAUSED" } else { "RUNNING" });
    }
}

fn handle_reset(
    mut params: ResMut<PhysicsParams>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    physics_query: Query<Entity, With<PhysicsObject>>,
) {
    if params.reset_scene {
        // Remove all physics objects
        for entity in physics_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Spawn new ones
        spawn_physics_objects(&mut commands, &mut meshes, &mut materials, 5);
        
        // Reset the flag
        params.reset_scene = false;
        
        println!("🔄 Scene reset!");
    }
}

fn spawn_random_objects(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    physics_query: Query<Entity, With<PhysicsObject>>,
) {
    // Spawn a new object every few seconds
    if time.elapsed_seconds() % 3.0 < 0.016 && physics_query.iter().len() < 10 {
        use bevy::math::*;
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-5.0..5.0);
        let z = rng.gen_range(-5.0..5.0);
        let color = Color::hsl(rng.gen_range(0.0..360.0), 0.8, 0.6);
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.3)),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                }),
                transform: Transform::from_xyz(x, 15.0, z),
                ..default()
            },
            PhysicsObject {
                velocity: Vec3::new(
                    rng.gen_range(-2.0..2.0),
                    0.0,
                    rng.gen_range(-2.0..2.0),
                ),
                mass: 1.0,
            },
            Bouncy,
        ));
    }
}

// ===== tests/integration.rs =====

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy_midi_params::prelude::*;

    #[derive(Resource, MidiParams, PartialEq)]
    struct TestParams {
        #[midi(1, 0.0..1.0)]
        pub value_a: f32,
        
        #[midi(2, -10.0..10.0)]
        pub value_b: f32,
        
        #[midi(33, button)]
        pub flag: bool,
    }

    impl Default for TestParams {
        fn default() -> Self {
            Self {
                value_a: 0.5,
                value_b: 0.0,
                flag: false,
            }
        }
    }

    #[test]
    fn test_midi_mappings_generation() {
        let mappings = TestParams::get_midi_mappings();
        assert_eq!(mappings.len(), 3);
        
        // Check range mapping
        let range_mapping = mappings.iter().find(|m| m.cc == 1).unwrap();
        assert_eq!(range_mapping.field_name, "value_a");
        assert_eq!(range_mapping.min_value, 0.0);
        assert_eq!(range_mapping.max_value, 1.0);
        
        // Check button mapping
        let button_mapping = mappings.iter().find(|m| m.cc == 33).unwrap();
        assert_eq!(button_mapping.field_name, "flag");
        assert!(matches!(button_mapping.control_type, crate::ControlType::Button));
    }

    #[test]
    fn test_midi_value_updates() {
        let mut params = TestParams::default();
        
        // Test range update
        let changed = params.update_from_midi(1, 0.5); // Normalized value
        assert!(changed);
        assert_eq!(params.value_a, 0.5);
        
        // Test button toggle
        let changed = params.update_from_midi(33, 0.8); // > 0.5 should toggle
        assert!(changed);
        assert_eq!(params.flag, true);
        
        // Test button not triggering on low value
        let changed = params.update_from_midi(33, 0.3);
        assert!(!changed);
        assert_eq!(params.flag, true); // Should remain true
    }

    #[test]
    fn test_persistence() {
        let params = TestParams {
            value_a: 0.75,
            value_b: 5.0,
            flag: true,
        };
        
        let persist_data = params.to_persist_data();
        assert!(persist_data.values.contains_key("value_a"));
        assert!(persist_data.values.contains_key("value_b"));
        assert!(persist_data.values.contains_key("flag"));
        
        let mut loaded_params = TestParams::default();
        loaded_params.from_persist_data(&persist_data);
        
        assert_eq!(loaded_params.value_a, 0.75);
        assert_eq!(loaded_params.value_b, 5.0);
        assert_eq!(loaded_params.flag, true);
    }

    #[test]
    fn test_plugin_setup() {
        let mut app = App::new();
        app.add_plugins(MidiParamsPlugin::new("test.ron").no_auto_connect());
        
        // Plugin should register the controller resource
        assert!(app.world.contains_resource::<MidiController>());
    }
}
