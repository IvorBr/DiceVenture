use bevy::prelude::*;
use rand::SeedableRng;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::island::*;
use crate::components::island_maps::IslandMaps;
use crate::components::island_maps::TerrainType;
use crate::components::overworld::{LocalIsland, Island};
use crate::plugins::network::MakeLocal;
use crate::components::player::LocalPlayer;
use crate::plugins::camera::NewCameraTarget;
use crate::components::player::Player;
use crate::plugins::network::OwnedBy;
use crate::preludes::network_preludes::*;
use crate::IslandSet;
use crate::GameState;
use rand::rngs::StdRng;
use rand::prelude::IndexedRandom;
use crate::islands::atoll::generate_atoll_tiles;

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
            (player_enters_island, player_leaves_island, detect_objective).run_if(server_running),
            (spawn_island_player, client_player_leaves_island).in_set(IslandSet)
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

fn client_setup_island(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_island: Query<(&Island, &IslandInfo), With<LocalIsland>>,
    mut islands: ResMut<IslandMaps>,
    client: Option<Res<RenetClient>>
) {
    let island_root = commands
        .spawn((
            IslandRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            InheritedVisibility::VISIBLE
        )).id();

    let (island, island_info) = local_island.single();

    //data stage
    let mut map = Map::new();
    let mut generator = StdRng::seed_from_u64(island.0);

    match island_info.island_type {
        IslandType::Atoll => generate_atoll_tiles(&mut map, island.0, &mut generator),
        IslandType::Forest => generate_atoll_tiles(&mut map, island.0, &mut generator),
        _ => Vec::new()
    };

    //visualize stage
    for (pos, chunk) in map.chunks.iter() {
        for (idx, tile) in chunk.tiles.iter().enumerate() {
            if !matches!(tile.kind, TileType::Terrain(_)) {
                continue;
            }
            let position = map.chunk_to_world_coords(*pos, idx);
            let mut color = Color::srgb(0.9, 0.8, 0.6);

            match tile.kind {
                TileType::Terrain(TerrainType::Sand) => {
                    color = Color::srgb(0.9, 0.8, 0.6);
                }
                TileType::Terrain(TerrainType::Rock) => {
                    color = Color::srgb(0.49, 0.51, 0.52);
                }
                TileType::Terrain(TerrainType::Boardwalk) => {
                    color = Color::srgb_u8(88, 57, 39);
                }
                TileType::Terrain(TerrainType::TreeTrunk) => {
                    color = Color::srgb_u8(88, 57, 39);
                }
                TileType::Terrain(TerrainType::Leaves) => {
                    color = Color::srgb_u8(36, 80, 2);
                }
                _ => ()
            }
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    ..Default::default()
                })),
                Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32),
            )).set_parent(island_root);
        }
    }

    let shoreline_tiles: Vec<IVec3> = map.shore_tiles();
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

    //TODO: Hacky fix, but now the island generation happens twice (server and in client)
    if client.is_some() {
        islands.maps.insert(island.0, map);
    }
}

fn server_setup_island(
    commands: &mut Commands,
    map: &mut Map,
    island: u64,
    island_type: IslandType,
) {
    let mut generator = StdRng::seed_from_u64(island);

    let top_tiles = match island_type {
        IslandType::Atoll => {
            generate_atoll_tiles(map, island, &mut generator)
        },
        _ => {
            let mut tiles: Vec<IVec3> = Vec::new();
            // fallback: simple square terrain
            for x in 0..16 {
                for z in 0..16 {
                    tiles.push(IVec3::new(x, 0, z));
                    map.add_entity_ivec3(
                        IVec3::new(x, 0, z),
                        Tile::new(TileType::Terrain(TerrainType::Sand), Entity::PLACEHOLDER),
                    );
                }
            }
            Vec::new()
        }
    };
    
    // spawn an enemy
    for _ in 0..5 {
        let enemy_pos = top_tiles.choose(&mut generator).unwrap().clone() + IVec3::Y;
        let enemy_id = commands
            .spawn((
                Enemy,
                EnemyState::Idle,
                Position(enemy_pos),
                EleminationObjective,
                MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating)),
                OnIsland(island),
                Aggression::RangeBased(5)
            ))
            .id();

        map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));
    }
}

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
    mut leave_island_event: EventReader<LeaveIsland>,
    mut islands: ResMut<IslandMaps>,
    client: Option<Res<RenetClient>>
) {   
    for event in leave_island_event.read() {
        if client.is_some() {
            islands.maps.remove(&event.0);
            println!("Deleting island on client");
        }
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
            leave_island_event.send(ToClients { mode: SendMode::Direct(owner.0), event: LeaveIsland(island.0) });    
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