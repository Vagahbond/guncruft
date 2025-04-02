use std::f32::consts::PI;

use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{color::palettes::css::BLUE, pbr::light_consts::lux::FULL_DAYLIGHT, prelude::*};
use player::{
    fps_camera::move_camera,
    fps_movement::{advance_fps_movement, handle_fps_movement, interpolate_fps_movement},
    player::create_player,
};

pub mod player;

#[derive(Component)]
struct Controlable;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_world, create_player))
        .add_systems(Update, (move_camera, animate_light))
        .add_systems(FixedUpdate, advance_fps_movement)
        .add_systems(
            // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
            RunFixedMainLoop,
            (
                // The physics simulation needs to know the player's input, so we run this before the fixed timestep loop.
                // Note that if we ran it in `Update`, it would be too late, as the physics simulation would already have been advanced.
                // If we ran this in `FixedUpdate`, it would sometimes not register player input, as that schedule may run zero times per frame.
                handle_fps_movement.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
                // The player's visual representation needs to be updated after the physics simulation has been advanced.
                // This could be run in `Update`, but if we run it here instead, the systems in `Update`
                // will be working with the `Transform` that will actually be shown on screen.
                interpolate_fps_movement.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ),
        )
        .run();
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    /* let mut primary_window = q_windows.single_mut().unwrap();

        // for a game that doesn't use the cursor (like a shooter):
        // use `Locked` mode to keep the cursor in one place
        primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;

        // also hide the cursor
        primary_window.cursor_options.visible = false;
    */
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            clearcoat: 1.0,
            clearcoat_perceptual_roughness: 0.1,
            metallic: 0.9,
            perceptual_roughness: 0.5,
            base_color: BLUE.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(0, 255, 0))),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            illuminance: FULL_DAYLIGHT,
            ..default()
        },
        Transform::from_xyz(-5.0, 10.0, -5.0).with_rotation(Quat::from_rotation_x(-PI / 4.)),
    ));

    // Chessboard Plane
    let black_material = materials.add(Color::BLACK);
    let white_material = materials.add(Color::WHITE);

    let plane_mesh = meshes.add(Plane3d::default().mesh().size(2.0, 2.0));

    for x in -3..4 {
        for z in -3..4 {
            commands.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(if (x + z) % 2 == 0 {
                    black_material.clone()
                } else {
                    white_material.clone()
                }),
                Transform::from_xyz(x as f32 * 2.0, -1.0, z as f32 * 2.0),
            ));
        }
    }
}

/// Moves the light around.
fn animate_light(
    mut lights: Query<&mut Transform, Or<(With<PointLight>, With<DirectionalLight>)>>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs();
    for mut transform in lights.iter_mut() {
        transform.translation = vec3(
            ops::sin(now * 1.4),
            ops::cos(now * 1.0),
            ops::cos(now * 0.6),
        ) * vec3(1.0, 4.0, 1.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
