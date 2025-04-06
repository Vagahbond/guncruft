use bevy::{
    ecs::system::Query,
    input::{ButtonInput, keyboard::KeyCode},
    prelude::*,
};

use crate::environment::cube::{CUBE_SIZE, Cube};

use super::fps_camera::FPSCamera;

#[derive(Component)]
pub struct CreativeMod;

pub fn lay_cube(
    mouse_input: Res<ButtonInput<MouseButton>>,
    object_query: Query<&Transform, (With<Cube>, Without<CreativeMod>)>, // Query for objects in the scene
    player_query: Query<(&mut Transform, &FPSCamera), (With<CreativeMod>, Without<Cube>)>,
) {
    if (mouse_input.pressed(MouseButton::Left)) {
        println!("Trying to lay cube");
        for (player_transform, _) in player_query.iter() {
            let origin = player_transform.translation; // Player's position
            let direction = player_transform.rotation * Vec3::Z; // Forward direction (Z-axis)

            let max_distance = 5.0; // Set the maximum distance for the ray

            // Check for intersections with objects
            for object_transform in object_query.iter() {
                if let Some(intersection) = ray_intersects_cube(
                    origin,
                    direction,
                    max_distance,
                    object_transform.translation,
                ) {
                    println!("Hit object at: {:?}", intersection);
                }
            }
        }
    }
}

fn ray_intersects_cube(
    ray_origin: Vec3,
    ray_direction: Vec3,
    max_distance: f32,
    cube_center: Vec3,
) -> Option<Vec3> {
    // Calculate the AABB corners
    let min = cube_center - CUBE_SIZE / 2.0;
    let max = cube_center + CUBE_SIZE / 2.0;

    // Initialize t_min and t_max for the intersection
    let mut t_min = (min.x - ray_origin.x) / ray_direction.x;
    let mut t_max = (max.x - ray_origin.x) / ray_direction.x;

    if t_min > t_max {
        std::mem::swap(&mut t_min, &mut t_max);
    }

    let mut ty_min = (min.y - ray_origin.y) / ray_direction.y;
    let mut ty_max = (max.y - ray_origin.y) / ray_direction.y;

    if ty_min > ty_max {
        std::mem::swap(&mut ty_min, &mut ty_max);
    }

    if (t_min > ty_max) || (ty_min > t_max) {
        return None; // No intersection
    }

    // Update t_min and t_max
    if ty_min > t_min {
        t_min = ty_min;
    }
    if ty_max < t_max {
        t_max = ty_max;
    }

    let mut tz_min = (min.z - ray_origin.z) / ray_direction.z;
    let mut tz_max = (max.z - ray_origin.z) / ray_direction.z;

    if tz_min > tz_max {
        std::mem::swap(&mut tz_min, &mut tz_max);
    }

    if (t_min > tz_max) || (tz_min > t_max) {
        return None; // No intersection
    }

    // Update t_min and t_max
    if tz_min > t_min {
        t_min = tz_min;
    }
    if tz_max < t_max {
        t_max = tz_max;
    }

    // Check if the intersection is within the max distance
    if t_min < 0.0 || t_min > max_distance {
        return None; // Intersection is behind the ray origin or beyond max distance
    }

    // Calculate the intersection point
    Some(ray_origin + ray_direction * t_min)
}
