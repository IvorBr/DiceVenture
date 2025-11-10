use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use crate::components::island_maps::TerrainType;

#[derive(Clone)]
pub struct TerrainInfo {
    pub scene: Handle<Scene>
}

#[derive(Resource)]
pub struct TerrainRegistry(pub HashMap<TerrainType, TerrainInfo>);

impl std::ops::Deref for TerrainRegistry {
    type Target = HashMap<TerrainType, TerrainInfo>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for TerrainRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl FromWorld for TerrainRegistry {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let mut map = HashMap::default();

        let mut insert = |tt: TerrainType, scene_path: &'static str| {
            map.insert(
                tt,
                TerrainInfo {
                    scene: assets.load(scene_path),
                },
            );
        };

        insert(
            TerrainType::Sand,
            "blocks/SandBlock.glb#Scene0",
        );
        insert(
            TerrainType::Rock,
            "blocks/RockBlock.glb#Scene0",
        );
        insert(
            TerrainType::Boardwalk,
            "blocks/BoardWalkBlock.glb#Scene0",
        );
        insert(
            TerrainType::PalmTree,
            "blocks/PalmTreeTrunkBlock.glb#Scene0",
        );

        TerrainRegistry(map)
    }
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainRegistry>();
    }
}
