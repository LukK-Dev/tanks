use std::f32::consts::PI;

use bevy::{prelude::*, utils::info, window::PrimaryWindow};
use bevy_xpbd_3d::{
    components::{
        CoefficientCombine, CollisionLayers, Friction, LayerMask, LinearVelocity, LockedAxes,
        RigidBody,
    },
    parry::na::ComplexField,
    plugins::spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use leafwing_input_manager::{action_state::ActionState, input_map::InputMap, InputManagerBundle};
use tanks::project_vector_onto_plane_y_axis;

use crate::game::{Action, GamePhysicsLayer};

#[derive(Resource, Default)]
struct AimPosition(Vec3);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AimPosition::default()).add_systems(
            Update,
            (
                movement,
                update_aim_position,
                aim_turret,
                update_last_frame_velocity,
            )
                .chain(),
        );
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
struct LastFrameVelocity(Vec3);

#[derive(Component, Default)]
struct TurnVelocity(f32);

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
    pub last_frame_velocity: LastFrameVelocity,
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
            last_frame_velocity: LastFrameVelocity::default(),
        }
    }
}

#[derive(Component)]
pub struct Turret;

#[derive(Bundle)]
pub struct TurretBundle {
    pub turret: Turret,
    pub pbr_bundle: PbrBundle,
}

fn movement(
    mut player: Query<
        (
            &mut Transform,
            &MovementParameters,
            &mut TurnVelocity,
            &mut LinearVelocity,
            &LastFrameVelocity,
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
    let (mut transform, params, mut turn_velocity, mut velocity, last_frame_velocity, action) =
        player.unwrap();
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

    // TODO: change turn direction when reversing
    // let velocity_delta = velocity.0 - last_frame_velocity.0;
    // let mut forward_sign = transform.forward().xz().dot(velocity_delta.xz()).signum();
    // if velocity_delta.length() < 0.004
    //     && !(action.pressed(&Action::TurnLeft) && action.pressed(&Action::TurnRight))
    // {
    //     forward_sign = 1.0;
    // }
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

fn aim_turret(
    mut turrets: Query<
        (&Parent, &mut Transform, &GlobalTransform),
        (With<Turret>, Without<Player>),
    >,
    players: Query<&Transform, With<Player>>,
    aim_position: Res<AimPosition>,
) {
    for (parent, mut turret_transform, global_turret_transform) in turrets.iter_mut() {
        let aim_position_adjusted_to_local_turret_space =
            aim_position.0 - global_turret_transform.translation();
        let player_transform = players
            .get(parent.get())
            .expect("Expected the turret to have a player.");
        let target = project_vector_onto_plane_y_axis(
            aim_position_adjusted_to_local_turret_space,
            turret_transform.local_y(),
            turret_transform.translation,
        );
        turret_transform.look_at(target, Vec3::Y);
        turret_transform.rotation *= player_transform.rotation.inverse();
    }
}

fn update_aim_position(
    mut aim_position: ResMut<AimPosition>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
) {
    if let Some((camera, camera_transform)) =
        cameras.iter().filter(|(camera, _)| camera.is_active).next()
    {
        let window = primary_window.get_single();
        if window.is_err() {
            return;
        }
        if let Some(cursor_position) = window.unwrap().cursor_position() {
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                if let Some(hit_info) = spatial_query.cast_ray(
                    ray.origin,
                    ray.direction,
                    100.0,
                    true,
                    SpatialQueryFilter::from_mask(GamePhysicsLayer::Ground),
                ) {
                    let hit_position = ray.origin + ray.direction * hit_info.time_of_impact;
                    aim_position.0 = hit_position;

                    cfg_if::cfg_if! {
                        if #[cfg(debug_assertions)] {
                            gizmos.sphere(hit_position, Quat::default(), 0.25, Color::RED);
                        }
                    }
                }
            }
        }
    }
}

fn update_last_frame_velocity(mut query: Query<(&LinearVelocity, &mut LastFrameVelocity)>) {
    for (velocity, mut last_frame_velocity) in query.iter_mut() {
        last_frame_velocity.0 = velocity.0;
    }
}
