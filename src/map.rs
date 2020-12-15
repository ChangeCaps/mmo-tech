use crate::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Tile {
    pub fn to_translation(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * 32.0,
            self.y as f32 * 32.0 + self.z as f32 * 16.0,
            0.0,
        )
    }
}

serde_sync!(Tile = 864817564917835867578123423412123);

#[derive(Default)]
pub struct Map {
    pub tiles: HashMap<Tile, Entity>,
}

#[derive(Serialize, Deserialize)]
pub struct TileSpawnable {
    tile: Tile,
}
network_uuid!(TileSpawnable = 56234786109236180564623789623789);

#[typetag::serde]
impl Spawnable for TileSpawnable {
    fn spawn(
        &self,
        commands: &mut Commands,
        resources: &Resources,
        _ctx: &SpawnContext,
        bundle: SpawnBundle,
    ) -> Entity {
        let mut map = resources.get_mut::<Map>().unwrap();

        let entity = commands
            .spawn(bundle)
            .with(Transform::default())
            .with(GlobalTransform::default())
            .with(self.tile.clone())
            .with(ComponentSync::<Tile>::ty(ActorTy::new::<Server>()))
            .current_entity()
            .unwrap();

        map.tiles.insert(self.tile.clone(), entity);

        entity
    }
}

pub fn tile_transform_system(mut query: Query<(&mut Transform, &Tile), Changed<Tile>>) {
    for (mut transform, tile) in query.iter_mut() {
        transform.translation = tile.to_translation();
    }
}
