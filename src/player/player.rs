use bevy::{
    asset::Assets,
    color::{
        Color,
        palettes::{css::WHITE, tailwind},
    },
    core_pipeline::core_3d::Camera3d,
    ecs::system::{Commands, ResMut},
    gizmos,
    math::{
        Vec3, Vec3Swizzles,
        primitives::{Cuboid, Sphere},
    },
    pbr::{MeshMaterial3d, NotShadowCaster, StandardMaterial},
    render::{
        camera::{Camera, PerspectiveProjection, Projection, Viewport},
        mesh::{Mesh, Mesh3d},
        view::RenderLayers,
    },
    transform::components::Transform,
    utils::default,
};

use super::{creative_mode::CreativeMod, fps_camera::FPSCamera, fps_movement::FPSMovement};

const DEFAULT_SENSITIVITY: f32 = 0.003;
/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;

pub fn create_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // TODO: something better than just a cuboid
    let arm = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));
    let cross_hair = meshes.add(Sphere::new(0.1));
    let cross_hair_material = materials.add(Color::from(tailwind::RED_950));

    // crosshair

    commands
        .spawn((
            Transform::default(),
            CreativeMod,
            FPSMovement {
                //prev_phys_translation: Vec3::new(-5.0, 1.80, -5.0),
                phys_translation: Vec3::new(-5.0, 1.80, 0.0),
                ..default()
            },
            FPSCamera {
                sensitivity: DEFAULT_SENSITIVITY,
            },
            Camera { ..default() },
            Camera3d { ..default() },
            Projection::from(PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }),
        ))
        .with_children(|parent| {
            // Spawn view model camera.
            parent.spawn((
                Camera3d::default(),
                Camera {
                    // Bump the order to render on top of the world model.
                    order: 1,
                    ..default()
                },
                Projection::from(PerspectiveProjection {
                    fov: 70.0_f32.to_radians(),
                    ..default()
                }),
                // Only render objects belonging to the view model.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            ));

            // Spawn the player's right arm.
            parent.spawn((
                Mesh3d(arm),
                MeshMaterial3d(arm_material),
                Transform::from_xyz(0.2, -0.1, -0.25),
                // Ensure the arm is only rendered by the view model camera.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                // The arm is free-floating, so shadows would look weird.
                NotShadowCaster,
            ));

            parent.spawn((
                Mesh3d(cross_hair),
                MeshMaterial3d(cross_hair_material),
                Transform::from_xyz(0.0, 0.0, -30.0),
                // Ensure the arm is only rendered by the view model camera.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                // The arm is free-floating, so shadows would look weird.
                NotShadowCaster,
            ));
        });
}
