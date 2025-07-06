use bevy::prelude::*;
use crate::components::island_maps::{Map};
use crate::components::island::GenerateIsland;
use crate::components::overworld::Island;

#[derive(Component)]
pub struct RENAMETHIS_ISLAND;

fn generate_island_map(
    mut island_maps: ResMut<IslandMaps>,
    new_islands: Query<&Island, (With<RENAMETHIS_ISLAND>, With<GenerateIsland>)>
) {
    for island_id in new_islands.iter() {
        //If island is already generated, return
        let mut generator = StdRng::seed_from_u64(island_id.0);

        let map = island_maps.maps.entry(island_id.0).or_insert_with(|| {
            let mut new_map = Map::new();
            generate_tiles(&mut new_map, island_id.0, &mut generator);
            new_map
        });
    }
}

fn generate_island_server(
    mut commands: Commands,
    mut island_maps: ResMut<IslandMaps>,
    islands: Query<(Entity, &Island), (With<RENAMETHIS_ISLAND>, With<MapFinishedIsland>)>,
) {
    for (entity, island_id) in islands.iter() {
        setup_island();
        commands.entity(entity).insert(FinishedSetupIsland).remove::<MapFinishedIsland>();
    }
}

pub struct RENAMETHIS_PLUGIN;
impl Plugin for RENAMETHIS_PLUGIN {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, // setup island tiles
            (generate_island_map.before(generate_island_server), 
            generate_island_server.run_if(server_running))
        );
    }
}
