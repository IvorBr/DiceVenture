use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use noise::Constant;
use noise::Exponent;
use noise::Multiply;
use noise::Perlin;
use noise::ScalePoint;
use noise::Simplex;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::island::*;
use crate::components::island_maps::IslandMaps;
use crate::components::overworld::{LocalIsland, Island};
use crate::plugins::network::MakeLocal;
use crate::components::player::LocalPlayer;
use crate::plugins::camera::NewCameraTarget;
use crate::components::player::Player;
use crate::plugins::network::OwnedBy;
use crate::preludes::network_preludes::*;
use crate::IslandSet;
use crate::GameState;
use rand::prelude::IndexedRandom;

use noise::{Fbm, NoiseFn};

pub struct IslandPlugin;
impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_client_event::<MoveDirection>(Channel::Ordered)
        .add_client_event::<EnteredIsland>(Channel::Unordered)
        .add_server_event::<LeaveIsland>(Channel::Unordered)
        .add_client_event::<AttackDirection>(Channel::Ordered)
        .add_server_event::<AttackAnimation>(Channel::Unreliable)
        .replicate::<OnIsland>()
        .replicate::<Player>()
        .replicate::<Position>()
        .add_systems(OnEnter(GameState::Island), client_setup_island)
        .add_systems(OnExit(GameState::Island), client_island_cleanup)
        .add_systems(PreUpdate, clean_up_island.run_if(server_running))
        .add_systems(Update, (
            (player_enters_island, player_leaves_island, clean_up_island, detect_objective).run_if(server_running),
            (spawn_island_player, client_player_leaves_island, input_regenerate_island).in_set(IslandSet)
        ));
    }
}

fn input_regenerate_island(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    island_root_query: Query<Entity, With<IslandRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.pressed(KeyCode::KeyR) {
        if let Ok(island_root) = island_root_query.get_single() {
            commands.entity(island_root).despawn_recursive();
        }

        let island_root = commands
            .spawn((
                IslandRoot,
                Transform::from_xyz(0.0, 0.0, 0.0),
                InheritedVisibility::VISIBLE,
            ))
            .id();

        let tiles = generate_atoll_tiles(rand::random());

        for tile in tiles.iter() {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(195, 180, 128),
                    ..Default::default()
                })),
                Transform::from_xyz(tile.x as f32, tile.y as f32, tile.z as f32),
            )).set_parent(island_root);
        }

        let shoreline_tiles: Vec<&IVec3> = find_shore_tiles(&tiles);

        for tile in shoreline_tiles.iter() {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.05, 1.05, 1.05))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 0.6),
                    ..Default::default()
                })),
                Transform::from_xyz(tile.x as f32, tile.y as f32, tile.z as f32),
            )).set_parent(island_root);
        }

        let min = tiles.iter().fold(IVec3::splat(i32::MAX), |a, b| a.min(*b));
        let max = tiles.iter().fold(IVec3::splat(i32::MIN), |a, b| a.max(*b));
        let center = (min + max) / 2;

        commands.spawn((
            NewCameraTarget,
            Transform::from_xyz(center.x as f32, center.y as f32 + 20.0, (center.z + max.z/2) as f32),
        ));
    }
}

fn client_island_cleanup(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    islandroot_query: Query<Entity, With<IslandRoot>>
) {
    //clean up visuals
    if let Ok(island_root) = islandroot_query.get_single() {
        commands.entity(island_root).despawn_recursive();
    }

    //clean up players
    for entity in player_query.iter() {
        commands.entity(entity).remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform)>();
    }

    //clean up enemies
    for entity in enemy_query.iter() {
        commands.entity(entity).remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform)>();
    }
}

fn spawn_island_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Position, Option<&LocalPlayer>, &OnIsland), (With<Player>, Without<Transform>)>,
    local_island: Query<&Island, With<LocalIsland>>,
) {
    for (entity, position, local, island) in players.iter() {
        if island.0 != local_island.single().0 {
            continue;
        }

        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(255, 255, 255),
                ..Default::default()
            })),
            Transform::from_xyz(
                position.0.x as f32,
                position.0.y as f32,
                position.0.z as f32
            ),
        ));

        if local.is_some() { 
            commands.entity(entity).insert(NewCameraTarget);
        }
    }
}

fn find_shore_tiles(tiles: &Vec<IVec3>) -> Vec<&IVec3> {
    let tile_set: HashSet<(i32, i32, i32)> = tiles.iter()
        .map(|t| (t.x, t.y, t.z))
        .collect();

    tiles.iter()
        .filter(|tile| {
            if tile.y != 0 { return false; }

            let neighbors = [
                (tile.x + 1, 0, tile.z),
                (tile.x - 1, 0, tile.z),
                (tile.x, 0, tile.z + 1),
                (tile.x, 0, tile.z - 1),
            ];
            neighbors.iter().any(|n| !tile_set.contains(n))
        })
        .collect()
}

fn client_setup_island(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_island: Query<(&Island, &IslandInfo), With<LocalIsland>>,
) {
    let island_root = commands
        .spawn((
            IslandRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            InheritedVisibility::VISIBLE
        )).id();

    let (seed, island_info) = local_island.single();

    let (color, tiles) = match island_info.island_type {
        IslandType::Atoll => (Color::srgb(0.9, 0.8, 0.6), generate_atoll_tiles(seed.0)),
        IslandType::Forest => (Color::srgb(0.0, 0.4, 0.0), generate_forest_tiles(seed.0)),
        _ => (Color::srgb(0.0, 0.4, 0.0), Vec::new()),
    };

    for tile in tiles.iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..Default::default()
            })),
            Transform::from_xyz(tile.x as f32, tile.y as f32, tile.z as f32),
        )).set_parent(island_root);
    }

    let shoreline_tiles: Vec<&IVec3> = find_shore_tiles(&tiles);

    for tile in shoreline_tiles.iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.05, 1.05, 1.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.6),
                ..Default::default()
            })),
            Transform::from_xyz(tile.x as f32, tile.y as f32, tile.z as f32),
        )).set_parent(island_root);
    }
}

fn server_setup_island(
    commands: &mut Commands,
    map: &mut Map,
    island: u64,
    island_type: IslandType,
) {
    let tiles = match island_type {
        IslandType::Atoll => {
            let tiles = generate_atoll_tiles(island);

            for tile in tiles.iter() {
                map.add_entity_ivec3(*tile, Tile::new(TileType::Terrain, Entity::PLACEHOLDER));
            }

            tiles
        },
        _ => {
            let mut tiles: Vec<IVec3> = Vec::new();
            // fallback: simple square terrain
            for x in 0..16 {
                for z in 0..16 {
                    tiles.push(IVec3::new(x, 0, z));
                    map.add_entity_ivec3(
                        IVec3::new(x, 0, z),
                        Tile::new(TileType::Terrain, Entity::PLACEHOLDER),
                    );
                }
            }

            tiles
        }
    };

    //setup leave position
    let shore = find_shore_tiles(&tiles);
    if let Some(spawn_tile) = shore.choose(&mut rand::rng()) {
        map.leave_position = **spawn_tile;
    }

    // spawn an enemy
    let enemy_pos = IVec3::new(5, 1, 1);
    let enemy_id = commands
        .spawn((
            Enemy { ..Default::default() },
            Position(enemy_pos),
            EleminationObjective,
            MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating)),
            OnIsland(island),
        ))
        .id();

    map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));
}

pub fn generate_atoll_tiles(seed: u64) -> Vec<IVec3> {
    let size = 50;
    let radius = size as f32 * 0.5;
    let center_offset = IVec3::new(8, 0, 8) - IVec3::new(size as i32 / 2, 0, size as i32 / 2);

    let mut base_noise = Fbm::<Perlin>::new(seed as u32);
    base_noise.octaves = 1;
    base_noise.frequency = 0.07;

    let terrain = base_noise;

    let mut tiles = Vec::new();
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

            if value > threshold {
                let mut height = ((value - threshold) * 10.0).ceil() as i32; //need to be optimized to only spawn seeable parts, can simply check for neighbours
                if height > 3 {
                    height = 3;
                }
                for y in 0..height {
                    tiles.push(IVec3::new(x, y, z) + center_offset);
                }
            } else {
                println!("shallow: {value}");
                tiles.push(IVec3::new(x, -1, z) + center_offset);
            }
        }
    }

    tiles
}

pub fn generate_forest_tiles(seed: u64) -> Vec<IVec3> {
    let size = 50;
    let radius = size as f32 * 0.5;
    let center_offset = IVec3::new(8, 0, 8) - IVec3::new(size as i32 / 2, 0, size as i32 / 2);

    let mut base_noise = Fbm::<Perlin>::new(seed as u32);
    base_noise.octaves = 4;
    base_noise.frequency = 0.07;

    let mut radial_falloff = Exponent::new(
        ScalePoint::new(Constant::new(1.0))
        .set_x_scale(1.0 / radius as f64)
        .set_z_scale(1.0 / radius as f64)
    );
    radial_falloff.exponent = 2.5;

    let terrain = Multiply::new(base_noise, radial_falloff);

    let mut tiles = Vec::new();
    let threshold = 0.1;

    for x in 0..size {
        for z in 0..size {
            let fx = x as f64;
            let fz = z as f64;

            let value = terrain.get([fx, fz]);

            if value > threshold {
                let height = ((value - threshold) * 10.0).ceil() as i32;

                for y in 0..height {
                    tiles.push(IVec3::new(x, y, z) + center_offset);
                }
            }
        }
    }

    tiles
}

// pub fn generate_atoll_tiles(seed: u64) -> Vec<IVec3> {
//     use noise::{Fbm, Perlin, NoiseFn};

//     let fbm: Fbm<Perlin> = Fbm::new(seed as u32);
//     let scale = 0.05;
//     let size = 50;
//     let threshold = 0.1;

//     let cx = size as f32 / 2.0;
//     let cz = size as f32 / 2.0;
//     let ring_radius = 8.0;
//     let ring_thickness = 6.0;

//     let mut tiles = Vec::new();

//     let center_offset = IVec3::new(8, 0, 8) - IVec3::new(size as i32 / 2, 0, size as i32 / 2);

//     for x in 0..size {
//         for z in 0..size {
//             let fx = x as f32;
//             let fz = z as f32;

//             let dx = fx - cx;
//             let dz = fz - cz;
//             let dist = (dx * dx + dz * dz).sqrt();

//             let ring = 1.0 - ((dist - ring_radius).abs() / ring_thickness).clamp(0.0, 1.0);
//             let noise = fbm.get([fx as f64 * scale, fz as f64 * scale]) as f32;

//             let base = noise * ring;

//             if base > threshold {
//                 let thickness = ((base - threshold) * 8.0).ceil() as i32;
//                 for y in 0..thickness {
//                     tiles.push(IVec3::new(x as i32, y as i32, z as i32) + center_offset);
//                 }
//             }
//         }
//     }

//     println!("tiles: {}", tiles.len());
//     tiles
// }

fn player_enters_island(
    mut commands: Commands,
    mut island_enter_event: EventReader<FromClient<EnteredIsland>>,
    mut islands: ResMut<IslandMaps>,
) {
    for FromClient { client_entity, event } in island_enter_event.read() {
        let island_id = event.0;

        let map = islands.maps.entry(island_id).or_insert_with(|| {
            let mut new_map = Map::new();
            server_setup_island(&mut commands, &mut new_map, island_id, IslandType::Atoll);
            new_map
        });
        
        let mut spawn_pos = map.leave_position;
        spawn_pos.y += 2;
        // Find an empty vertical position above the default spawn
        while map.get_tile(spawn_pos).kind != TileType::Empty {
            spawn_pos.y += 1;
        }

        println!("Spawning player: {:?} on island {:?}", client_entity, island_id);

        let player_entity = commands.spawn((
            Player,
            Position(spawn_pos),
            OwnedBy(*client_entity),
            OnIsland(island_id),
        )).id();

        commands.server_trigger_targets(
            ToClients {
                mode: SendMode::Direct(*client_entity),
                event: MakeLocal,
            },
            player_entity,
        );

        map.add_player(spawn_pos, player_entity);
    }
}

fn detect_objective(
    target_query: Query<&EleminationObjective>
) {
    if target_query.is_empty() {
       // reward player
    }
}

fn client_player_leaves_island(
    mut state: ResMut<NextState<GameState>>,
    leave_island_event: EventReader<LeaveIsland>
) {    
    if leave_island_event.len() > 0 {
        state.set(GameState::Overworld);
    }
}

fn player_leaves_island(
    mut commands: Commands,
    player_query: Query<(&Position, Entity, &OwnedBy, &OnIsland), With<Player>>,
    mut islands: ResMut<IslandMaps>,
    mut leave_island_event: EventWriter<ToClients<LeaveIsland>>,
) {
    for (position, entity, owner, island) in &player_query {
        let map = islands.get_map_mut(island.0);

        if position.0 == map.leave_position + IVec3::Y {
            println!("{:?} leaves island", entity);

            commands.entity(entity).try_despawn_recursive();
            map.remove_entity(position.0);
            map.player_count -= 1;
            leave_island_event.send(ToClients { mode: SendMode::Direct(owner.0), event: LeaveIsland });    
        }
    }
}

fn clean_up_island(
    mut commands: Commands,
    mut islands: ResMut<IslandMaps>,
    enemy_query: Query<(Entity, &OnIsland), With<Enemy>>,
) {
    islands.maps.retain(|id, map| {
        if map.player_count == 0 {
            println!("No players left on island {:?}: cleaning up", id);
            
            for (enemy_entity, island_id) in enemy_query.iter() {
                if *id == island_id.0 {
                    commands.entity(enemy_entity).despawn_recursive();

                }
            }
    
            false
        } else {
            true
        }
    });
    
}

// fn update_island(
//     mut map_events: EventReader<MapUpdate>,
//     mut map: ResMut<Map>,
//     mut meshes: ResMut<Assets<Mesh>>, 
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut commands: Commands,
//     islandroot_query: Query<Entity, With<IslandRoot>>
// ) {
//     if let Ok(island_root) = islandroot_query.get_single() {
//         for event in map_events.read() {
//             match event.0 {
//                 UpdateType::LoadTerrain => {
//                     let terrain_id = commands.spawn((
//                         Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
//                         MeshMaterial3d(materials.add(StandardMaterial {
//                             base_color: Color::srgb_u8(100, 255, 100),
//                             ..Default::default()
//                         })),
//                         Transform::from_xyz(event.1.x as f32, 0.0, event.1.z as f32)
//                     )).set_parent(island_root)
//                     .id();
                    
//                     //event.3 has the tile type, currently hard coded...
//                     map.add_entity_ivec3(event.1, Tile::new(TileType::Terrain, terrain_id));
//                 }
//                 UpdateType::UnloadTerrain => {
//                     if let Some(chunk) = map.chunks.get(&event.1) {
//                         let mut entities_to_despawn = Vec::new();
        
//                         for tile in &chunk.tiles {
//                             if commands.get_entity(tile.entity).is_some() && tile.kind == TileType::Terrain { //BIG PROBLEM CURRENTLY WITH ENEMY REPLICATION!!! check trello
//                                 entities_to_despawn.push(tile.entity);
//                             }
//                         }
                    
//                         for entity in entities_to_despawn {
//                             commands.entity(entity).despawn();
//                         }
                        
//                         map.chunks.remove(&event.1);
//                     }
//                 }
//             }   
//         }
//     }
// }

