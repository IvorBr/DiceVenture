
use bevy::prelude::*;
use noise::Perlin;
use rand::seq::IndexedRandom;
use crate::attacks::base_attack::BaseAttack;
use crate::components::humanoid::{AttackCooldowns, Position};
use crate::components::enemy::{Attacks, Enemy, EnemyState, MoveTimer, RangeAggro, KNIGHT_MOVE, STANDARD, STANDARD_MOVE};
use crate::components::island::{FinishedSetupIsland, GenerateIsland, MapFinishedIsland, OnIsland};
use crate::components::overworld::Island;
use crate::plugins::attack::{key_of, AttackSpec};
use crate::preludes::network_preludes::*;

use rand::rngs::StdRng;
use rand::SeedableRng;

use rand::Rng;
use noise::{Fbm, NoiseFn};
use crate::components::island_maps::{Map, IslandMaps, TerrainType};
use crate::islands::core::{add_boardwalk, reserve_with_margin};


#[derive(Component)]
pub struct Atoll;

pub struct AtollPlugin;
impl Plugin for AtollPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, // setup island tiles
            (generate_island_map.before(generate_island_server), 
            generate_island_server.run_if(server_running))
        );
    }
}

fn generate_island_map(
    mut commands: Commands,
    mut island_maps: ResMut<IslandMaps>,
    new_islands: Query<(Entity, &Island), (With<Atoll>, With<GenerateIsland>)>
) {
    for (entity, island_id) in new_islands.iter() {
        let mut generator = StdRng::seed_from_u64(island_id.0);
        let map = island_maps.maps.get(&island_id.0);

        if !map.is_some() {
            let mut new_map = Map::new();
            generate_tiles(&mut new_map, island_id.0, &mut generator);
            island_maps.maps.insert(island_id.0, new_map);
            commands.entity(entity).insert(MapFinishedIsland).remove::<GenerateIsland>();
            println!("Added island to the maps");
        }
        else {
            commands.entity(entity).remove::<GenerateIsland>();
        }
    }
}

fn generate_island_server(
    mut commands: Commands,
    mut island_maps: ResMut<IslandMaps>,
    islands: Query<(Entity, &Island), (With<Atoll>, With<MapFinishedIsland>)>,
) {
    for (entity, island_id) in islands.iter() {
        let map = island_maps.maps.get_mut(&island_id.0).unwrap();
        let mut generator = StdRng::seed_from_u64(island_id.0);
        
        let top_tiles = map.above_water_top_tiles();
        for _ in 0..5 {
            let enemy_pos = top_tiles.choose(&mut generator).unwrap().clone() + IVec3::Y;
            let enemy_id = commands
                .spawn((
                    Enemy,
                    STANDARD_MOVE,
                    Attacks(vec![key_of::<BaseAttack>()]),
                    AttackCooldowns::default(),
                    EnemyState::Idle,
                    Position(enemy_pos),
                    MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating)),
                    OnIsland(island_id.0),
                    RangeAggro(5)
                ))
                .id();

            map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));
        }

        commands.entity(entity).insert(FinishedSetupIsland).remove::<MapFinishedIsland>();
    }
}

pub fn generate_tiles(map: &mut Map, seed: u64, generator: &mut StdRng) -> Vec<IVec3> {
    let size = 50;
    let radius = size as f32 * 0.5;
    let center_offset = IVec3::new(8, 0, 8) - IVec3::new(size as i32 / 2, 0, size as i32 / 2);

    let mut base_noise = Fbm::<Perlin>::new(seed as u32);
    base_noise.octaves = 1;
    base_noise.frequency = 0.07;

    let terrain = base_noise;

    let threshold = 0.0;

    for x in 0..size {
        for z in 0..size {
            let fx = x as f64;
            let fz = z as f64;

            let mut value = terrain.get([fx, fz]);

            let dx = x as f32 - size as f32 / 2.0;
            let dz = z as f32 - size as f32 / 2.0;
            let distance = (dx * dx + dz * dz).sqrt() / radius;
            value -= distance.powf(2.5) as f64 - 0.2;
            let mut tile;

            if value > threshold {
                let mut height = ((value - threshold) * 10.0).ceil() as i32; //need to be optimized to only spawn seeable parts, can simply check for neighbours
                if height > 3 {
                    height = 3;
                }
                for y in 0..height {
                    tile = IVec3::new(x, y, z) + center_offset;
                    map.add_entity_ivec3(tile, Tile::new(TileType::Terrain(TerrainType::Sand), Entity::PLACEHOLDER));
                }
            } else {
                tile = IVec3::new(x, value.floor() as i32, z) + center_offset;
                map.add_entity_ivec3(tile, Tile::new(TileType::Terrain(TerrainType::Sand), Entity::PLACEHOLDER));

            }
        }
    }

    let top_tiles = map.above_water_top_tiles();
    let mut reserved_positions = HashSet::new();

    add_boardwalk(map, &mut reserved_positions, generator);

    //add trees
    let num_trees = generator.random_range(3..=6);
    for _ in 0..num_trees {
        if let Some(&base) = top_tiles.choose(generator) {
            if reserved_positions.contains(&base) { continue; }

            let height = generator.random_range(2..=4);
            let mut tree_positions = vec![];

            // Trunk
            for i in 0..height {
                tree_positions.push(base + IVec3::new(0, i, 0));
            }

            // Leaves
            let leaf_size = generator.random_range(2..=3);
            let leaf_min = -(leaf_size / 2);
            let leaf_max = leaf_min + leaf_size - 1;

            for dx in leaf_min..=leaf_max {
                for dy in 0..=1 {
                    for dz in leaf_min..=leaf_max {
                        tree_positions.push(base + IVec3::new(dx, height + dy, dz));
                    }
                }
            }

            if tree_positions.iter().any(|p| reserved_positions.contains(p)) {
                continue;
            }
            
            for pos in tree_positions.iter() {
                let terrain = if pos.y < base.y + height {
                    TerrainType::TreeTrunk
                } else {
                    TerrainType::Leaves
                };

                map.add_entity_ivec3(
                    *pos,
                    Tile::new(TileType::Terrain(terrain), Entity::PLACEHOLDER),
                );
            }

            reserve_with_margin(&mut reserved_positions, &tree_positions, 1);
        }
    }

    //add rocks
    let num_boulders = generator.random_range(0..=4);
    for _ in 0..num_boulders {
        if let Some(&center) = top_tiles.choose(generator) {
            if reserved_positions.contains(&center) { continue; }

            let radius: i32 = generator.random_range(1..=3);
            let size = if radius == 1 { 2 } else { radius + 2 };
            let min = -(size / 2);
            let max = min + size - 1;

            let mut rock_positions = vec![];

            for dx in min..=max {
                for dy in min..=max {
                    for dz in min..=max {
                        let mut extremes = 0;
                        if dx == min || dx == max { extremes += 1; }
                        if dy == min || dy == max { extremes += 1; }
                        if dz == min || dz == max { extremes += 1; }

                        if radius > 1 && extremes >= 2 {
                            continue;
                        }

                        let offset = IVec3::new(dx, dy, dz);
                        let pos = center + offset + IVec3::Y;
                        rock_positions.push(pos);
                    }
                }
            }

            if rock_positions.iter().any(|p| reserved_positions.contains(p)) {
                continue;
            }

            for pos in rock_positions.iter() {
                map.add_entity_ivec3(
                    *pos,
                    Tile::new(TileType::Terrain(TerrainType::Rock), Entity::PLACEHOLDER),
                );
            }

            reserve_with_margin(&mut reserved_positions, &rock_positions, 1);
        }
    }

    top_tiles
}