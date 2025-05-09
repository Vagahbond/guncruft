use std::f32::consts::PI;

use bevy::{
    ecs::{component::Component, system::Query},
    input::{ButtonInput, keyboard::KeyCode},
    math::{Vec2, Vec3},
    prelude::*,
};

#[derive(Component, Default)]
pub struct FPSMovement {
    /// A vector representing the player's input, accumulated over all frames that ran
    /// since the last time the physics simulation was advanced.
    pub acc_input: Vec2,
    /// A vector representing the player's velocity in the physics simulation.
    pub velocity: Vec3,
    /// The actual position of the player in the physics simulation.
    /// This is separate from the `Transform`, which is merely a visual representation.
    ///
    /// If you want to make sure that this component is always initialized
    /// with the same value as the `Transform`'s translation, you can
    /// use a [component lifecycle hook](https://docs.rs/bevy/0.14.0/bevy/ecs/component/struct.ComponentHooks.html)
    pub phys_translation: Vec3,
    /// The value [`PhysicalTranslation`] had in the last fixed timestep.
    /// Used for interpolation in the `interpolate_rendered_transform` system.
    pub prev_phys_translation: Vec3,
}

/// Handle keyboard input and accumulate it in the `AccumulatedInput` component.
///
/// There are many strategies for how to handle all the input that happened since the last fixed timestep.
/// This is a very simple one: we just accumulate the input and average it out by normalizing it.
pub fn handle_fps_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut FPSMovement)>,
) {
    const SPEED: f32 = 2.0;
    for (transform, mut mov) in query.iter_mut() {
        let forward = -Vec2::new(transform.forward().x, transform.forward().z);
        let right = Vec2::new(transform.forward().z, -transform.forward().x);

        if keyboard_input.pressed(KeyCode::KeyW) {
            mov.acc_input -= forward;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            mov.acc_input += forward;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            mov.acc_input += right;
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            mov.acc_input -= right;
        }

        // Need to normalize and scale because otherwise
        // diagonal movement would be faster than horizontal or vertical movement.
        // This effectively averages the accumulated input.
        let mut normalized = mov.acc_input.extend(0.0).normalize_or_zero() * SPEED;

        if (keyboard_input.pressed(KeyCode::ShiftLeft)) {
            normalized *= 2.0;
        }
        mov.velocity.x = normalized.x;
        mov.velocity.z = normalized.y;
    }
}

pub fn interpolate_fps_movement(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &FPSMovement)>,
) {
    for (mut transform, mov) in query.iter_mut() {
        let previous = mov.prev_phys_translation;
        let current = mov.phys_translation;
        // The overstep fraction is a value between 0 and 1 that tells us how far we are between two fixed timesteps.
        let alpha = fixed_time.overstep_fraction();

        let rendered_translation = previous.lerp(current, alpha);
        transform.translation = rendered_translation;
    }
}

/// Advance the physics simulation by one fixed timestep. This may run zero or multiple times per frame.
///
/// Note that since this runs in `FixedUpdate`, `Res<Time>` would be `Res<Time<Fixed>>` automatically.
/// We are being explicit here for clarity.
pub fn advance_fps_movement(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut FPSMovement, &Camera)>,
) {
    for (mut mov, camera) in query.iter_mut() {
        camera.hdr;
        mov.prev_phys_translation = mov.phys_translation;
        let new_translation = mov.velocity * fixed_time.delta_secs();
        // println!("{}", new_translation);
        mov.phys_translation += new_translation;

        // Reset the input accumulator, as we are currently consuming all input that happened since the last fixed timestep.
        mov.acc_input = Vec2::ZERO;
    }
}
