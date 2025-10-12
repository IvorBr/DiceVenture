
use bevy::prelude::*;
use noise::Perlin;
use rand::random_range;
use rand::seq::IndexedRandom;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::island::EliminationObjective;
use crate::components::island::EnterPosition;
use crate::components::island::OnIsland;
use crate::attacks::base_attack::BaseAttack;
use crate::components::island::{CompletedIslandObjective, FinishedSetupIsland, GenerateIsland, MapFinishedIsland};
use crate::components::overworld::Island;
use crate::components::overworld::OverworldIslandVisual;
use crate::plugins::attack::key_of;
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
            (generate_island_overworld,
            generate_island_map.before(setup_island_server), 
            setup_island_server.run_if(server_running))
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

fn setup_island_server(
    mut commands: Commands,
    mut island_maps: ResMut<IslandMaps>,
    islands: Query<(Entity, &Island, Option<&CompletedIslandObjective>), (With<Atoll>, With<MapFinishedIsland>)>,
) {
    for (entity, island_id, island_obj) in islands.iter() {

        if island_obj.is_none() {
            let map = island_maps.maps.get_mut(&island_id.0).unwrap();
            let mut generator = StdRng::seed_from_u64(island_id.0);
            
            let top_tiles = map.above_water_top_tiles();
            for _ in 0..4 {
                let enemy_pos = top_tiles.choose(&mut generator).unwrap().clone() + IVec3::Y;
                let enemy_id = commands
                    .spawn((
                        Enemy,
                        STANDARD_MOVE,
                        Attacks(vec![key_of::<BaseAttack>()]),
                        AttackCooldowns::default(),
                        EnemyState::Idle,
                        Position::new(enemy_pos),
                        MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating), false),
                        OnIsland(island_id.0),
                        RangeAggro(8),
                        Damage { value: 10 }
                    ))
                    .id();

                map.add_enemy(enemy_pos, enemy_id);
                commands.entity(entity).insert(EliminationObjective);
            }
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
                let mut height = ((value - threshold) * 10.0).ceil() as i32; //TODO: need to be optimized to only spawn seeable parts, can simply check for neighbours
                if height > 3 {     // no tiles higher than 3 on islands
                    height = 3;
                }
                for y in 0..height { // fill in tiles below above tile
                    tile = IVec3::new(x, y, z) + center_offset;
                    map.add_entity_ivec3(tile, Tile::new(TileType::Terrain(TerrainType::Sand), Entity::PLACEHOLDER));
                }
            } else { // underwater terrain
                let height = (value * 10.0).ceil() as i32;
                if height > -5 {
                    for y in 0 ..= 1 {
                    tile = IVec3::new(x, height - y, z) + center_offset;
                    map.add_entity_ivec3(tile, Tile::new(TileType::Terrain(TerrainType::Sand), Entity::PLACEHOLDER)); //TODO: do the neighbour check here too
                }
                }
            }
        }
    }

    let top_tiles = map.above_water_top_tiles();
    let mut reserved_positions = HashSet::new();

    add_boardwalk(map, &mut reserved_positions, generator);

    //add trees
    let num_trees = generator.random_range(5..=10);
    for _ in 0..num_trees {
        if let Some(&base) = top_tiles.choose(generator) {
            if reserved_positions.contains(&base) { continue; }
            let base_pos = base + IVec3::Y;
            let height = generator.random_range(3..=4);
            let mut tree_positions = vec![];

            for i in 0..height {
                tree_positions.push(base_pos + IVec3::new(0, i, 0));
            }

            if tree_positions.iter().any(|p| reserved_positions.contains(p)) {
                continue;
            }
            
            for pos in tree_positions.iter() {
                let terrain = if pos.y != base_pos.y + (height)/2 - 1 {
                    TerrainType::Invisible
                } else {
                    TerrainType::PalmTree
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

pub fn generate_island_overworld(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    new_islands: Query<(Entity, &Island, &Transform), Added<Atoll>>,
) {
    let island_size = 1.0;
    for (entity, island_id, tf) in new_islands.iter() {
        let mut generator = StdRng::seed_from_u64(island_id.0);

        let mut map = Map::new();
        generate_tiles(&mut map, island_id.0, &mut generator);

        let parent = commands.spawn((OverworldIslandVisual, Transform::from_translation(Vec3::ZERO), Visibility::Visible)).id();
        commands.entity(entity).add_child(parent).insert(EnterPosition(map.leave_position.as_vec3()/island_size + tf.translation));

        for (chunk_pos, chunk) in map.chunks.iter() {
            for (tile_index, tile) in chunk.tiles.iter().enumerate() {
                let mut color: Color;
                match tile.kind {
                    TileType::Terrain(TerrainType::Sand) => {
                        color = Color::srgb(0.9, 0.8, 0.6);
                    }
                    TileType::Terrain(TerrainType::Rock) => {
                        color = Color::srgb(0.49, 0.51, 0.52);
                    }
                    TileType::Terrain(TerrainType::Boardwalk) => {
                        color = Color::srgb_u8(100, 60, 40);
                    }
                    TileType::Terrain(TerrainType::PalmTree) => {
                        color = Color::srgb_u8(88, 57, 39);

                        let child = commands.spawn((
                            Mesh3d(meshes.add(Cuboid::new(1.0/island_size, 1.0/island_size, 1.0/island_size))),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgb_u8(36, 80, 2),
                                ..Default::default()
                            })),
                            Transform::from_translation((map.chunk_to_world_coords(*chunk_pos, tile_index) + IVec3::Y).as_vec3()/island_size),
                            Visibility::Inherited
                        )).id();

                        commands.entity(parent).add_child(child);
                    }
                    TileType::Terrain(TerrainType::TreeTrunk) => {
                        color = Color::srgb_u8(88, 57, 39);
                    }
                    TileType::Terrain(TerrainType::Leaves) => {
                        color = Color::srgb_u8(36, 80, 2);
                    }
                    _ => continue,
                }
                
                let jitter = random_range(-0.05..=0.05);
                let [r0, g0, b0, a] = color.to_linear().to_f32_array();
                let r = (r0 + jitter).clamp(0.0, 1.0);
                let g = (g0 + jitter).clamp(0.0, 1.0);
                let b = (b0 + jitter).clamp(0.0, 1.0);

                color = Color::srgba(r, g, b, a);
                let child = commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0/island_size, 1.0/island_size, 1.0/island_size))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        ..Default::default()
                    })),
                    Transform::from_translation((map.chunk_to_world_coords(*chunk_pos, tile_index)).as_vec3()/island_size),
                    Visibility::Inherited
                )).id();

                commands.entity(parent).add_child(child);
            }
            
        }
    }
}