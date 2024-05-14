use std::ops::Neg;

use bevy::prelude::*;

trait TwistAngle {
    fn twist_angle(&self, axis: Direction3d) -> f32;
}

// source: Luke Hutchison, https://stackoverflow.com/questions/3684269/component-of-a-quaternion-rotation-around-an-axis
impl TwistAngle for Quat {
    fn twist_angle(&self, axis: Direction3d) -> f32 {
        let rotation_axis = self.xyz();
        let dot_product = axis.dot(rotation_axis);
        let projection = axis * dot_product;
        let mut twist =
            Quat::from_xyzw(projection.x, projection.y, projection.z, self.w).normalize();
        0.0
    }
}

pub fn project_vector_onto_plane_y_axis(
    vector: Vec3,
    plane_normal: Direction3d,
    plane_position: Vec3,
) -> Vec3 {
    let d = plane_normal.dot(plane_position);
    let projected_y = (d - plane_normal.x * vector.x - plane_normal.z * vector.z) / plane_normal.y;
    return Vec3::new(vector.x, projected_y, vector.z);
}
