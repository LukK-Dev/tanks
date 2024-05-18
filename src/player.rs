use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_xpbd_3d::components::{
    CoefficientCombine, CollisionLayers, Friction, LayerMask, LinearVelocity, LockedAxes, RigidBody,
};
use leafwing_input_manager::{action_state::ActionState, input_map::InputMap, InputManagerBundle};

use crate::{
    game::{Action, GamePhysicsLayer},
    turret::TurretPlugin,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TurretPlugin).add_systems(Update, movement);
    }
}

#[derive(Component)]
pub struct MovementParameters {
    pub acceleration: f32,
    pub dampening: f32,
    pub max_velocity: f32,
    pub turn_acceleration: f32,
    pub turn_dampening: f32,
    pub max_turn_velocity: f32,
}

impl Default for MovementParameters {
    fn default() -> Self {
        Self {
            acceleration: 3.0,
            dampening: 10.0,
            max_velocity: 2.0,
            turn_acceleration: 20.0,
            turn_dampening: 15.0,
            max_turn_velocity: PI * 0.5,
        }
    }
}

#[derive(Component, Default)]
pub struct TurnVelocity(f32);

#[derive(Component, Default)]
pub struct Player;

// collider can be added as child
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub pbr_bundle: PbrBundle,
    pub input_bundle: InputManagerBundle<Action>,
    pub movement_parameters: MovementParameters,
    pub collision_layers: CollisionLayers,
    pub rigidbody: RigidBody,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
    pub turn_velocity: TurnVelocity,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        let mut input_map = InputMap::new([
            (Action::TurnLeft, KeyCode::KeyA),
            (Action::TurnRight, KeyCode::KeyD),
            (Action::MoveForward, KeyCode::KeyW),
            (Action::MoveBackward, KeyCode::KeyS),
        ]);
        input_map.insert(Action::Shoot, MouseButton::Left);

        Self {
            player: Player,
            pbr_bundle: PbrBundle::default(),
            input_bundle: InputManagerBundle::with_map(input_map),
            movement_parameters: MovementParameters::default(),
            collision_layers: CollisionLayers::new(GamePhysicsLayer::Default, LayerMask::ALL),
            rigidbody: RigidBody::Dynamic,
            locked_axes: LockedAxes::default(),
            friction: Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            turn_velocity: TurnVelocity::default(),
        }
    }
}

fn movement(
    mut player: Query<
        (
            &mut Transform,
            &MovementParameters,
            &mut TurnVelocity,
            &mut LinearVelocity,
            &ActionState<Action>,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    #[cfg(debug_assertions)] mut gizmos: Gizmos,
) {
    let player = player.get_single_mut();
    if player.is_err() {
        return;
    }
    let (mut transform, params, mut turn_velocity, mut velocity, action) = player.unwrap();
    let dt = time.delta_seconds();
    let mut forward_sign = 1.0;

    match (
        action.pressed(&Action::MoveForward),
        action.pressed(&Action::MoveBackward),
    ) {
        (true, false) => velocity.0 += transform.forward() * params.acceleration * dt,
        (false, true) => {
            forward_sign = -1.0;
            velocity.0 -= transform.forward() * params.acceleration * dt
        }
        _ => {
            let dampening_factor = 1.0 - params.dampening * dt;
            velocity.0.x *= dampening_factor;
            velocity.0.z *= dampening_factor;
        }
    }
    let clamped_horizontal_velocity = velocity.xz().clamp_length_max(params.max_velocity);
    let projected_horizontal_velocity =
        clamped_horizontal_velocity.project_onto(transform.forward().xz());
    velocity.x = projected_horizontal_velocity.x;
    velocity.z = projected_horizontal_velocity.y;

    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            gizmos.arrow(
                transform.translation,
                transform.translation + velocity.0,
                Color::RED,
            );
        }
    }

    match (
        action.pressed(&Action::TurnLeft),
        action.pressed(&Action::TurnRight),
    ) {
        (true, false) => turn_velocity.0 += params.turn_acceleration * forward_sign * dt,
        (false, true) => turn_velocity.0 -= params.turn_acceleration * forward_sign * dt,
        _ => turn_velocity.0 *= 1.0 - params.turn_dampening * dt,
    }
    turn_velocity.0 = turn_velocity
        .0
        .clamp(-params.max_turn_velocity, params.max_turn_velocity);
    transform.rotate_local_y(turn_velocity.0 * dt);
}
