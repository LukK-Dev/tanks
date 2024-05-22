use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                app.add_systems(Update, debug_draw_projectile_trajectories);
            }
        }
    }
}

#[derive(Component)]
pub struct Projectile {
    pub projectile_type: ProjectileType,
    pub velocity: f32,
}

impl Projectile {
    fn calculate_angle_to_hit_target(
        &self,
        launch_position: Vec3,
        target_position: Vec3,
        g: f32,
    ) -> Option<f32> {
        let x = launch_position.xz().distance(target_position.xz());
        let y = launch_position.y - target_position.y;
        let discriminant = self.velocity.powi(4) - g.powi(2) * x + 2.0 * y * self.velocity.powi(2);
        if discriminant == 0.0 {
            return None;
        }
        let angle = (self.velocity.powi(2) + discriminant.sqrt()) / g * x;
        Some(angle)
    }
}

#[derive(Default)]
pub enum ProjectileType {
    #[default]
    Standard,
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub pbr_bundle: PbrBundle,
    pub rigid_body: RigidBody,
    pub collider: Collider,
}

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        fn debug_draw_projectile_trajectories(mut gizmos: Gizmos) {}
    }
}
