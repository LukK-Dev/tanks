use bevy::{prelude::*, window::PrimaryWindow};
use bevy_xpbd_3d::plugins::spatial_query::{SpatialQuery, SpatialQueryFilter};
use tanks::project_vector_onto_plane_y_axis;

use crate::{game::GamePhysicsLayer, player::Player};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AimPosition(Vec3::default()))
            .add_systems(Update, (update_aim_position, aim_turret).chain());
    }
}

#[derive(Resource, Default)]
struct AimPosition(Vec3);

#[derive(Component)]
pub struct Turret;

#[derive(Bundle)]
pub struct TurretBundle {
    pub turret: Turret,
    pub pbr_bundle: PbrBundle,
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
