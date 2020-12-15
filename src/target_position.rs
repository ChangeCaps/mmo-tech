use crate::*;
use serde::*;

#[derive(Serialize, Deserialize)]
pub struct TargetPosition {
    pub position: Vec2,
}

serde_sync!(TargetPosition = 3461874691536481231254314);

impl TargetPosition {
    pub fn new(position: Vec2) -> Self {
        Self { position }
    }
}

pub fn target_position_system(mut query: Query<(&TargetPosition, &mut Transform)>) {
    for (target_position, mut transform) in query.iter_mut() {
        let pos_a = transform.translation.truncate();
        let pos_b = target_position.position;

        let dist = (pos_a - pos_b).length();

        if dist > 0.1 {
            transform.translation = ((pos_a * 3.0 + pos_b) / 4.0).extend(transform.translation.z);
        }
    }
}
