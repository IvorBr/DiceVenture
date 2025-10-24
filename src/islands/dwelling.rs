
use bevy::prelude::*;
use noise::Perlin;
use rand::seq::IndexedRandom;

use crate::components::core::Seed;
use crate::components::island::IslandGen;
use crate::components::island::{CompletedIslandObjective, FinishedSetupIsland, MapFinishedIsland};
use crate::components::island_maps::Grid;
use crate::plugins::enterables::{generate_island_overworld, generate_island_map};
use crate::preludes::network_preludes::*;

use rand::rngs::StdRng;
use rand::SeedableRng;

use rand::Rng;
use noise::{Fbm, NoiseFn};
use crate::components::island_maps::{TerrainType};
use crate::islands::core::{add_boardwalk, reserve_with_margin};

#[derive(Component)]
pub struct Dwelling;

impl IslandGen for Dwelling {
    fn generate_tiles(map: &mut Grid, seed: u64, rng: &mut StdRng) {
        generate_tiles(map, seed, rng);
    }
}

pub struct DwellingPlugin;
impl Plugin for DwellingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, // setup island tiles
        (
            (generate_island_map::<Dwelling>, 
            setup_island_server.run_if(in_state(ServerState::Running))).chain(),
            generate_island_overworld::<Dwelling>)
        );
    }
}

fn setup_island_server(
    mut commands: Commands,
    mut islands: Query<(Entity, &mut Grid, Option<&CompletedIslandObjective>, &Seed), (With<Dwelling>, With<MapFinishedIsland>)>,
) {
    for (entity, mut grid, island_obj, seed) in islands.iter_mut() {
        if island_obj.is_none() {
            let mut generator = StdRng::seed_from_u64(seed.0);
        }

        commands.entity(entity).insert(FinishedSetupIsland).remove::<MapFinishedIsland>();
    }
}

pub fn generate_tiles(map: &mut Grid, seed: u64, generator: &mut StdRng) -> Vec<IVec3> {
    let size = 50;

    let top_tiles = map.above_water_top_tiles();

    top_tiles
}

