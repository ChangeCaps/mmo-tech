use bevy::prelude::*;

pub fn z_sort_system(
    mut query: Query<&mut Transform, Without<bevy::render::camera::Camera>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation.z = transform.translation.y * -0.00000001;
    }
}