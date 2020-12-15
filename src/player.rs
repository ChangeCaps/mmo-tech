use crate::*;
use network::*;

#[derive(Serialize, Deserialize)]
pub struct MovementDirection {
    direction: Vec2,
}
serde_sync!(MovementDirection = 2658768195371452387452783612347);

#[derive(Serialize, Deserialize)]
pub struct MovementSpeed(pub f32);
serde_sync!(MovementSpeed = 46197851341234126834145234623594234);

#[derive(Serialize, Deserialize)]
pub struct PlayerSpawnable {
    actor_id: ActorId,
}

#[derive(Default)]
pub struct Player {
    pub entity: Option<Entity>,
    pub camera: Option<Entity>,
}

#[typetag::serde]
impl Spawnable for PlayerSpawnable {
    fn spawn(
        &self,
        commands: &mut Commands,
        resources: &Resources,
        ctx: &SpawnContext,
        bundle: SpawnBundle,
    ) -> Entity {
        let mut animator = Animator::new();
        animator.add_animation("walk_up_left", Animation::new(0..=23, 1.0 / 12.0));
        animator.add_animation("walk_left", Animation::new(24..=47, 1.0 / 12.0));
        animator.add_animation("walk_down_left", Animation::new(48..=71, 1.0 / 12.0));
        animator.add_animation("walk_down", Animation::new(72..=95, 1.0 / 12.0));
        animator.add_animation("walk_down_right", Animation::new(96..=119, 1.0 / 12.0));
        animator.add_animation("walk_right", Animation::new(120..=143, 1.0 / 12.0));
        animator.add_animation("walk_up_right", Animation::new(144..=167, 1.0 / 12.0));
        animator.add_animation("walk_up", Animation::new(168..=191, 1.0 / 12.0));

        let entity = commands
            .spawn(bundle)
            .with(Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)))
            .with(MovementDirection {
                direction: Vec2::new(0.0, 0.0),
            })
            .with(TargetPosition::new(Vec2::zero()))
            .with(MovementSpeed(60.0))
            .with(animator.build())
            .with(ComponentSync::<MovementDirection>::id(self.actor_id))
            .with(ComponentSync::<TargetPosition>::id(ctx.sender_id()))
            .with(ComponentSync::<Animator>::ty(ActorTy::new::<Server>()))
            .current_entity()
            .unwrap();

        if ctx.local_ty().is::<Client>() {
            let asset_server = resources.get::<AssetServer>().unwrap();
            let mut texture_atlases = resources.get_mut::<Assets<TextureAtlas>>().unwrap();
            let mut player = resources.get_mut::<Player>().unwrap();

            let texture_handle = asset_server.load("sheet.png");
            let texture_atlas = texture_atlases.add(TextureAtlas::from_grid(
                texture_handle,
                Vec2::new(128.0, 128.0),
                24*8,
                1,
            ));

            commands
                .with_bundle(SpriteSheetBundle {
                    texture_atlas,
                    ..Default::default()
                });

            if ctx.local_id() == self.actor_id {
                player.entity = Some(entity);
            }
        }

        entity
    }
}

pub fn player_input_system(
    input: Res<Input<KeyCode>>,
    player: Res<Player>,
    mut query: Query<&mut MovementDirection>,
) {
    if let Some(entity) = &player.entity {
        if let Ok(mut movement_direction) = query.get_mut(*entity) {
            let mut direction = Vec2::zero();

            if input.pressed(KeyCode::W) {
                direction.y += 1.0;
            }

            if input.pressed(KeyCode::S) {
                direction.y -= 1.0;
            }

            if input.pressed(KeyCode::D) {
                direction.x += 1.0;
            }

            if input.pressed(KeyCode::A) {
                direction.x -= 1.0;
            }

            if direction != movement_direction.direction {
                movement_direction.direction = direction;
            }
        }
    }
}

const MOVEMENT_FRAMES: [bool; 24] = [
    true, // 0
    true, // 1
    true, // 2
    true, // 3
    true, // 4
    false, // 5
    false, // 6
    false, // 7
    false, // 8
    true, // 9
    true, // 10
    true, // 11
    true, // 12
    true, // 13
    true, // 14
    true, // 15
    true, // 16
    false, // 17
    false, // 18
    false, // 19
    false, // 20
    true, // 21
    true, // 22
    true, // 23
];

pub fn player_movement_system(
    time: Res<Time>,
    mut query: Query<(
        &MovementDirection,
        &MovementSpeed,
        &mut TargetPosition,
        &mut Animator,
    )>,
) {
    for (movement_direction, movement_speed, mut target_position, mut animator) in query.iter_mut()
    {
        if movement_direction.direction.length() != 0.0 {
            let direction = movement_direction.direction.normalize();

            if MOVEMENT_FRAMES[animator.current_frame()] {
                target_position.position += direction * movement_speed.0 * time.delta_seconds();
            }

            let angle = direction.y.atan2(direction.x);
            let x = angle / std::f32::consts::PI * 4.0 - 0.5;

            if x > 3.0 {
                animator.set_playing("walk_left");
            } else if x > 2.0 {
                animator.set_playing("walk_up_left");
            } else if x > 1.0 {
                animator.set_playing("walk_up");
            } else if x > 0.0 {
                animator.set_playing("walk_up_right");
            } else if x > -1.0 {
                animator.set_playing("walk_right");
            } else if x > -2.0 {
                animator.set_playing("walk_down_right");
            } else if x > -3.0 {
                animator.set_playing("walk_down");
            } else if x > -4.0 {
                animator.set_playing("walk_down_left");
            }
        }
    }
}

pub fn player_spawn_system(
    mut network_handle: ResMut<NetworkHandle>,
    mut event_reader: Local<EventReader<ConnectionEvent>>,
    events: Res<Events<ConnectionEvent>>,
) {
    for event in event_reader.iter(&events) {
        if let ConnectionEvent::Connected { actor, .. } = event {
            let player = PlayerSpawnable {
                actor_id: actor.id(),
            };

            network_handle.spawn(NetworkTarget::All, player);
        }
    }
}

pub fn player_camera_system(player: Res<Player>, mut query: Query<&mut Transform>) {
    if player.entity.is_some() && player.camera.is_some() {
        let player_position = query
            .get_mut(player.entity.unwrap())
            .unwrap()
            .translation
            .truncate();
        let mut camera_transform = query.get_mut(player.camera.unwrap()).unwrap();
        camera_transform.translation = player_position.extend(camera_transform.translation.z);
    }
}
