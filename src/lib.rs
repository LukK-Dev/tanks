use bevy::prelude::*;

pub fn project_vector_onto_plane_y_axis(
    vector: Vec3,
    plane_normal: Direction3d,
    plane_position: Vec3,
) -> Vec3 {
    let d = plane_normal.dot(plane_position);
    let projected_y = (d - plane_normal.x * vector.x - plane_normal.z * vector.z) / plane_normal.y;
    return Vec3::new(vector.x, projected_y, vector.z);
}

pub fn cycle_fullscreen_on_f11(
    mut primary_window: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::F11) {
        let mut window = primary_window
            .get_single_mut()
            .expect("Expected a Window to exist.");

        window.mode = match window.mode {
            bevy::window::WindowMode::Windowed => bevy::window::WindowMode::Fullscreen,
            _ => bevy::window::WindowMode::Windowed,
        }
    }
}
