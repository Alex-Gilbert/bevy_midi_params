use bevy::prelude::*;
use bevy_midi_params::{MidiControllable, MidiParams, MidiParamsPlugin};

#[derive(Resource, MidiParams)]
struct MeshController {
    // Integer control - selects which mesh to display (0-4)
    #[midi(16, 0..4)]
    pub mesh_index: i32,

    pub prev_mesh_index: usize,

    // Color control - each component gets its own CC
    #[midi(20)] // red
    #[midi(21)] // green
    #[midi(22)] // blue
    #[midi(23)] // alpha
    pub mesh_color: LinearRgba,

    // Vector control - position offset
    #[midi(24, -5.0..5.0)] // x offset
    #[midi(25, -5.0..5.0)] // y offset
    #[midi(26, -5.0..5.0)] // z offset
    pub position_offset: Vec3,

    // Boolean toggle - visibility
    #[midi(58)]
    pub visible: bool,

    // Float range - rotation speed
    #[midi(19, 0.0..10.0)]
    pub rotation_speed: f32,
}

impl Default for MeshController {
    fn default() -> Self {
        Self {
            mesh_index: 0,
            prev_mesh_index: 0,
            mesh_color: LinearRgba::rgb(1.0, 0.5, 0.3),
            position_offset: Vec3::ZERO,
            visible: true,
            rotation_speed: 1.0,
        }
    }
}

#[derive(Component)]
struct MeshDisplay {
    meshes: Vec<Handle<Mesh>>,
}

#[derive(Component)]
struct RotatingMesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            MidiParamsPlugin::default()
                .with_controller("midi mix"),
        ) // Add the MIDI plugin
        .init_resource::<MeshController>() // Initialize our MIDI-controlled resource
        .add_systems(Startup, setup)
        .add_systems(Update, (update_mesh_display, rotate_mesh))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Create 5 different mesh types
    let mesh_handles = vec![
        meshes.add(Cuboid::new(2.0, 2.0, 2.0)),
        meshes.add(Sphere::new(1.5).mesh().ico(5).unwrap()),
        meshes.add(Cylinder::new(1.0, 2.0)),
        meshes.add(Torus::new(1.0, 0.5)),
        meshes.add(Capsule3d::new(0.5, 2.0)),
    ];

    commands.spawn((
        Mesh3d(mesh_handles[0].clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::LinearRgba(LinearRgba::rgb(1.0, 0.5, 0.3)),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MeshDisplay {
            meshes: mesh_handles,
        },
        RotatingMesh,
    ));
}

fn update_mesh_display(
    mut controller: ResMut<MeshController>,
    mut query: Query<(
        &mut Mesh3d,
        &mut Transform,
        &mut Visibility,
        &MeshMaterial3d<StandardMaterial>,
        &MeshDisplay,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut mesh, mut transform, mut visibility, mat_handle, display) in &mut query {
        // Update mesh based on index (wraps around if out of bounds)
        let index = (controller.mesh_index as usize) % display.meshes.len();
        if controller.prev_mesh_index != index {
            mesh.0 = display.meshes[index].clone();
            controller.prev_mesh_index = index;
        }

        // Update position offset
        transform.translation = controller.position_offset;

        // Update visibility
        *visibility = if controller.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // Update material color
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.base_color = Color::LinearRgba(controller.mesh_color);
        }
    }
}

fn rotate_mesh(
    time: Res<Time>,
    controller: Res<MeshController>,
    mut query: Query<&mut Transform, With<RotatingMesh>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(controller.rotation_speed * time.delta_secs());
    }
}

// Usage in main app:
// 1. Connect a MIDI controller
// 2. Map CC 16 to switch between meshes (cube, sphere, cylinder, torus, capsule)
// 3. Map CC 20-23 to control RGB color
// 4. Map CC 30-32 to move the mesh in 3D space
// 5. Map CC 40 to toggle visibility
// 6. Map CC 50 to control rotation speed

// The macro automatically generates:
// - update_from_midi() method that handles CC messages
// - get_midi_mappings() for the plugin to know what to listen for
// - Proper type handling for i32, Vec3, LinearRgba, bool, and f32
