use crate::features::camera::AccumulatedInput;
use crate::features::position::{PreviousWorldPosition, WorldPosition};
use crate::features::states::AppState;
use bevy::app::RunFixedMainLoopSystem::AfterFixedMainLoop;
use bevy::prelude::*;

pub struct MovementPlugin;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default)]
pub struct Acceleration(pub Vec2);

#[derive(Component, Default)]
pub struct Friction(pub f32);

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                apply_acceleration,
                apply_velocity,
                reset_input,
                apply_friction,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        // .add_systems(Update, interpolate_rendered_transform);
        .add_systems(
            RunFixedMainLoop,
            interpolate_rendered_transform
                .in_set(AfterFixedMainLoop)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

fn apply_acceleration(mut query: Query<(&mut Velocity, &Acceleration)>, time: Res<Time<Fixed>>) {
    for (mut velocity, acceleration) in &mut query {
        velocity.0 += acceleration.0 * time.delta_secs();
    }
}

fn apply_velocity(
    mut query: Query<(&mut WorldPosition, &mut PreviousWorldPosition, &Velocity)>,
    time: Res<Time<Fixed>>,
) {
    for (mut world_position, mut previous_world_position, velocity) in &mut query {
        previous_world_position.0 = world_position.0;

        world_position.x += velocity.0.x * time.delta_secs();
        world_position.y += velocity.0.y * time.delta_secs();
    }
}

fn reset_input(mut query: Query<&mut AccumulatedInput>) {
    for mut input in &mut query {
        input.0 = Vec2::ZERO;
    }
}

fn apply_friction(mut query: Query<(&Friction, &mut Velocity)>, time: Res<Time<Fixed>>) {
    for (friction, mut velocity) in &mut query {
        let friction_direction = -velocity.0.normalize_or_zero();
        let friction_magnitude = velocity.0.length() * velocity.0.length();
        let friction_vector = friction_direction * friction_magnitude;
        let friction_coefficient = friction.0;

        // TODO: This is to stop camera from gliding for too long - this is SILLY
        const min_velocity: f32 = 2.0;

        velocity.0 += friction_coefficient * friction_vector * time.delta_secs();
        if velocity.0.length() < min_velocity {
            velocity.0 = Vec2::ZERO;
        }
    }
}

fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &WorldPosition, &PreviousWorldPosition)>,
) {
    for (mut transform, current_world_position, previous_world_position) in query.iter_mut() {
        let previous = previous_world_position.0;
        let current = current_world_position.0;
        // The overstep fraction is a value between 0 and 1 that tells us how far we are between two fixed timesteps.
        let alpha = fixed_time.overstep_fraction();

        //println!("previous vs current: {:?}", (previous, current));
        let rendered_translation = previous.lerp(current, alpha);
        transform.translation.x = rendered_translation.x;
        transform.translation.z = rendered_translation.y;
    }
}
