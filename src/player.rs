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
        app.insert_resource(AimPosition::default())
            .add_systems(Update, (movement, update_aim_position, aim_turret).chain());
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
            acceleration: 2.0,
            dampening: 10.0,
            max_velocity: 2.0,
            turn_acceleration: 20.0,
            turn_dampening: 15.0,
            max_turn_velocity: PI * 0.5,
        }
    }
}

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
    pub turn_velocity: TurnVelocity,
    pub collision_layers: CollisionLayers,
    pub rigidbody: RigidBody,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
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
            turn_velocity: TurnVelocity::default(),
            collision_layers: CollisionLayers::new(GamePhysicsLayer::Default, LayerMask::ALL),
            rigidbody: RigidBody::Dynamic,
            locked_axes: LockedAxes::default(),
            friction: Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
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

    match (
        action.pressed(&Action::TurnLeft),
        action.pressed(&Action::TurnRight),
    ) {
        (true, false) => turn_velocity.0 += params.turn_acceleration * dt,
        (false, true) => turn_velocity.0 -= params.turn_acceleration * dt,
        _ => turn_velocity.0 *= 1.0 - params.turn_dampening * dt,
    }
    turn_velocity.0 = turn_velocity
        .0
        .clamp(-params.max_turn_velocity, params.max_turn_velocity);
    transform.rotate_local_y(turn_velocity.0 * dt);

    // let velocity_delta = transform.forward() * params.acceleration * dt;
    // if action.pressed(&Action::MoveForward) {
    //     velocity.0 += velocity_delta
    // }
    // if action.pressed(&Action::MoveBackward) {
    //     velocity.0 -= velocity_delta
    // }
    // velocity.0 = transform.forward() * velocity.0.length();
    //
    // velocity.0 = velocity.0.clamp_length_max(params.max_velocity);
    //
    // if !action.pressed(&Action::MoveForward) && !action.pressed(&Action::MoveBackward) {
    //     velocity.0.x *= params.dampening;
    //     velocity.0.z *= params.dampening;
    // }

    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            gizmos.arrow(
                transform.translation,
                transform.translation + velocity.0,
                Color::RED,
            );
        }
    }
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
