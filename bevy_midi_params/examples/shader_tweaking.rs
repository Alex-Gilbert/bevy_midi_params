//! Advanced example for shader artists and technical artists.
//! 
//! This example shows how to use MIDI controllers for real-time material
//! and shader parameter tweaking - perfect for rapid iteration.
//! 
//! Controls (AKAI MIDImix):
//! - Knob 1: Roughness (0.0 - 1.0)
//! - Knob 2: Metallic (0.0 - 1.0)  
//! - Knob 3: Emission strength (0.0 - 3.0)
//! - Knob 4: Normal map strength (0.0 - 2.0)
//! - Knob 5: Subsurface (0.0 - 1.0)
//! - Knob 6: Clearcoat (0.0 - 1.0)

use bevy::prelude::*;
use bevy_midi_params::prelude::*;

#[derive(Resource, MidiParams)]
struct MaterialParams {
    #[midi(1, 0.0..1.0)]
    pub roughness: f32,
    
    #[midi(2, 0.0..1.0)]
    pub metallic: f32,
    
    #[midi(3, 0.0..3.0)]
    pub emission_strength: f32,
    
    #[midi(4, 0.0..2.0)]
    pub normal_strength: f32,
    
    #[midi(5, 0.0..1.0)]
    pub subsurface: f32,
    
    #[midi(6, 0.0..1.0)]
    pub clearcoat: f32,
}

impl Default for MaterialParams {
    fn default() -> Self {
        Self {
            roughness: 0.5,
            metallic: 0.0,
            emission_strength: 0.0,
            normal_strength: 1.0,
            subsurface: 0.0,
            clearcoat: 0.0,
        }
    }
}

#[derive(Resource, MidiParams)]
struct LightingParams {
    #[midi(7, 0.0..5000.0)]
    pub light_intensity: f32,
    
    #[midi(8, -180.0..180.0)]
    pub light_rotation: f32,
    
    #[midi(9, 0.0..1.0)]
    pub ambient_strength: f32,
}

impl Default for LightingParams {
    fn default() -> Self {
        Self {
            light_intensity: 3000.0,
            light_rotation: 45.0,
            ambient_strength: 0.2,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MidiParamsPlugin::new("shader_tweaking.ron"))
        .add_systems(Startup, setup_shader_scene)
        .add_systems(Update, (
            update_materials,
            update_lighting,
        ))
        .run();
}

fn setup_shader_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create multiple objects with different base materials
    let mesh = meshes.add(Sphere::new(1.0));
    
    // Different base materials to test parameters
    let positions = [
        (Vec3::new(-3.0, 0.0, 0.0), Color::WHITE),      // White sphere
        (Vec3::new(-1.0, 0.0, 0.0), Color::RED),        // Red sphere
        (Vec3::new(1.0, 0.0, 0.0), Color::GREEN),       // Green sphere
        (Vec3::new(3.0, 0.0, 0.0), Color::BLUE),        // Blue sphere
    ];
    
    for (position, color) in positions {
        commands.spawn((
            PbrBundle {
                mesh: mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                }),
                transform: Transform::from_translation(position),
                ..default()
            },
            TweakableMaterial,
        ));
    }

    // Ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10.0, 10.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.8, 0.8),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -2.0, 0.0),
        ..default()
    });

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Directional light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 3000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -0.5)),
            ..default()
        },
        TweakableLight,
    ));
}

#[derive(Component)]
struct TweakableMaterial;

#[derive(Component)]
struct TweakableLight;

fn update_materials(
    params: Res<MaterialParams>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&Handle<StandardMaterial>, With<TweakableMaterial>>,
) {
    if !params.is_changed() {
        return;
    }

    for material_handle in material_query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.perceptual_roughness = params.roughness;
            material.metallic = params.metallic;
            
            // Apply emission with color based on strength
            let emission_color = Color::rgb(
                params.emission_strength * 0.8,
                params.emission_strength * 0.4,
                params.emission_strength * 0.2,
            );
            material.emissive = emission_color;
            
            // Note: Normal strength and other advanced parameters would require
            // custom shaders or materials in a real implementation
        }
    }
    
    // Print current values for reference
    println!("🎨 Material Parameters:");
    println!("  Roughness: {:.3}", params.roughness);
    println!("  Metallic: {:.3}", params.metallic);
    println!("  Emission: {:.3}", params.emission_strength);
    println!("  Normal: {:.3}", params.normal_strength);
}

fn update_lighting(
    params: Res<LightingParams>,
    mut light_query: Query<(&mut DirectionalLight, &mut Transform), With<TweakableLight>>,
) {
    if !params.is_changed() {
        return;
    }

    for (mut light, mut transform) in light_query.iter_mut() {
        light.illuminance = params.light_intensity;
        
        // Rotate light based on parameter
        let rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            params.light_rotation.to_radians(),
            -0.5,
        );
        transform.rotation = rotation;
    }
    
    println!("💡 Lighting Parameters:");
    println!("  Intensity: {:.0} lux", params.light_intensity);
    println!("  Rotation: {:.1}°", params.light_rotation);
    println!("  Ambient: {:.3}", params.ambient_strength);
}
