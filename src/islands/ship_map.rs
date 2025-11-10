
use bevy::prelude::*;
use noise::Perlin;
use rand::seq::IndexedRandom;

use crate::components::core::Seed;
use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::island::EliminationObjective;
use crate::components::island::IslandGen;
use crate::components::island::OnIsland;
use crate::attacks::base_attack::BaseAttack;
use crate::components::island::{CompletedIslandObjective, FinishedSetupIsland, MapFinishedIsland};
use crate::components::island_maps::Grid;
use crate::components::overworld::Island;
use crate::components::ship::Ship;
use crate::plugins::attack::key_of;
use crate::plugins::enterables::{generate_island_overworld, generate_island_map};
use crate::preludes::network_preludes::*;

use rand::rngs::StdRng;
use rand::SeedableRng;

use rand::Rng;
use noise::{Fbm, NoiseFn};
use crate::components::island_maps::TerrainType;
use crate::islands::core::{add_boardwalk, reserve_with_margin};


impl IslandGen for Ship {
    fn generate_tiles(map: &mut Grid, seed: u64, rng: &mut StdRng) {
        generate_tiles(map, seed, rng);
    }
}

pub struct ShipMapPlugin;
impl Plugin for ShipMapPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, // setup island tiles
        (
            generate_island_map::<Ship>.before(setup_island_server), 
            setup_island_server.run_if(in_state(ServerState::Running)),
            generate_island_overworld::<Ship>)
        );
    }
}

fn setup_island_server(
    mut commands: Commands,
    mut islands: Query<(Entity, &mut Grid, Option<&CompletedIslandObjective>, &Seed), (With<Ship>, With<MapFinishedIsland>)>,
) {
    for (entity, mut grid, island_obj, seed) in islands.iter_mut() {
        if island_obj.is_none() {
            let mut generator = StdRng::seed_from_u64(seed.0);
            
            let top_tiles = grid.above_water_top_tiles();
            // for _ in 0..4 {
            //     let enemy_pos = top_tiles.choose(&mut generator).unwrap().clone() + IVec3::Y;
            //     let enemy_id = commands
            //         .spawn((
            //             Enemy,
            //             STANDARD_MOVE,
            //             Attacks(vec![key_of::<BaseAttack>()]),
            //             AttackCooldowns::default(),
            //             EnemyState::Idle,
            //             Position::new(enemy_pos),
            //             MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating), false),
            //             OnIsland(entity),
            //             RangeAggro(8),
            //             Damage { value: 10 }
            //         ))
            //         .id();

            //     grid.add_enemy(enemy_pos, enemy_id);
            //     commands.entity(entity).insert(EliminationObjective);
            // }
        }

        commands.entity(entity).insert(FinishedSetupIsland).remove::<MapFinishedIsland>();
    }
}

pub fn generate_tiles(map: &mut Grid, seed: u64, generator: &mut StdRng) -> Vec<IVec3> {
    let size = 3;

    for x in -1..size -1 {
        for z in -1..size -1 {
            map.add_entity_ivec3(IVec3::new(x, 0, z), Tile::new(TileType::Terrain(TerrainType::Invisible), Entity::PLACEHOLDER));
        }
    }   

    let top_tiles = map.above_water_top_tiles();

    top_tiles
}

