use bevy::prelude::*;
use crate::components::island_maps::{Map};

#[derive(Component)]
pub struct RENAMETHIS_ISLAND;

fn generate_island_map(
    mut island_maps: ResMut<Map>,
    new_islands: Query<IslandId, With<RENAMETHIS_ISLAND, GenerateIsland>>
) {
    //If island is already generated, return
    for IslandId in new_islands.iter() {
        if islands.maps.entry(island_id) {
            return;
        }
    }
    
}

fn generate_island_server(
    mut commands: Commands,
    mut island_maps: ResMut<Map>,
    setup_islands: Query<RENAMETHIS_ISLAND, With<SetupIsland>>,
) {
   
}

pub struct RENAMETHIS_PLUGIN;
impl Plugin for RENAMETHIS_PLUGIN {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, // setup island tiles
            generate_island_map.before(setup_dynamic), 
            generate_island_server.run_if(server_running)
        );
    }
}
