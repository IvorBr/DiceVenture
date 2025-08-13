use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::island::*;
use crate::components::island_maps::IslandMaps;
use crate::components::island_maps::TerrainType;
use crate::components::overworld::{LocalIsland, Island};
use crate::islands::atoll::AtollPlugin;
use crate::plugins::network::MakeLocal;
use crate::components::character::LocalPlayer;
use crate::plugins::camera::NewCameraTarget;
use crate::components::character::Character;
use crate::plugins::network::OwnedBy;
use crate::preludes::network_preludes::*;
use crate::IslandSet;
use crate::GameState;

pub struct IslandPlugin;
impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(AtollPlugin)
        .add_client_event::<EnteredIsland>(Channel::Unordered)
        .add_server_event::<LeaveIsland>(Channel::Unordered)
        .replicate::<OnIsland>()
        .replicate::<Character>()
        .replicate::<Position>()
        .add_systems(OnExit(GameState::Island), client_island_cleanup)
        .add_systems(PreUpdate, ((clean_up_island, add_waiting_player).run_if(server_running), visualize_island))
        .add_systems(Update, (
            (player_enters_island, player_leaves_island, elimination_island_objective).run_if(server_running),
            (spawn_island_player, client_player_leaves_island).in_set(IslandSet)
        ));
    }
}

fn client_island_cleanup(
    mut commands: Commands,
    player_query: Query<Entity, With<Character>>,
    enemy_query: Query<Entity, With<Enemy>>,
    islandroot_query: Query<Entity, With<IslandRoot>>
) {
    //clean up visuals
    if let Ok(island_root) = islandroot_query.single() {
        commands.entity(island_root).despawn();
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
    players: Query<(Entity, &Position, Option<&LocalPlayer>, &OnIsland), (With<Character>, Without<Transform>)>,
    local_island_query: Query<&Island, With<LocalIsland>>,
) {
    for (entity, position, local, island) in players.iter() {
        if let Ok(local_island) = local_island_query.single() {
            if island.0 != local_island.0 {
                continue;
            }
        }

        commands.entity(entity).insert((
            Transform::from_xyz(
                position.0.x as f32,
                position.0.y as f32,
                position.0.z as f32
            ),
        ));

        let visual = commands.spawn((
            VisualEntity,
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(255, 255, 255),
                ..Default::default()
            })),
        ))
        .id();

        commands.entity(entity)
            .add_child(visual)
            .insert(VisualRef(visual));

        if local.is_some() { 
            commands.entity(entity).insert(NewCameraTarget);
        }
    }
}

fn visualize_island(
    mut commands: Commands,
    islands: Query<(Entity, &Island), (Without<GenerateIsland>, With<VisualizeIsland>)>,
    mut island_maps: ResMut<IslandMaps>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
){    
    if let Ok((entity, island)) = islands.single() {
        let island_root = commands
        .spawn((
            IslandRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            InheritedVisibility::VISIBLE
        )).id();
        let map = island_maps.maps.get_mut(&island.0).unwrap();
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
                )).insert(ChildOf(island_root));
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
            )).insert(ChildOf(island_root));
        }
        commands.entity(entity).remove::<VisualizeIsland>();
    }
}  

fn add_waiting_player(
    mut commands: Commands,
    players: Query<(Entity, &OnIsland), With<Waiting>>,
    mut islands: ResMut<IslandMaps>,
) {
    for (entity, island) in players.iter() {
        if let Some(map) = islands.maps.get_mut(&island.0) {
            let mut spawn_pos = map.leave_position;
            spawn_pos.y += 2;
            while map.get_tile(spawn_pos).kind != TileType::Empty {
                spawn_pos.z += 1;
            }

            commands.entity(entity).insert(Position(spawn_pos)).remove::<Waiting>();

            map.add_player(spawn_pos, entity);
        }
    }
}

fn player_enters_island(
    mut commands: Commands,
    mut island_enter_event: EventReader<FromClient<EnteredIsland>>,
    islands: Query<(Entity, &Island)>,
) {
    for FromClient { client_entity, event } in island_enter_event.read() {
        let island_id = event.0;
        let player_entity = commands.spawn((
            Character,
            Health::new(200),
            OwnedBy(*client_entity),
            Waiting,
            OnIsland(island_id),
        )).id();
        
        commands.server_trigger_targets(
            ToClients {
                mode: SendMode::Direct(*client_entity),
                event: MakeLocal,
            },
            player_entity,
        );

        for (entity, island) in islands.iter() {
            if island.0 == island_id {
                commands.entity(entity).insert(GenerateIsland);
            }
        }
        println!("ADDING PLAYER {:?}, WAITING FOR ISLAND", player_entity);
    }
}

fn elimination_island_objective(
    mut commands: Commands,
    target_query: Query<(Entity, &Island), (With<FinishedSetupIsland>, With<EliminationObjective>, Without<CompletedIslandObjective>)>,
    mut island_maps: ResMut<IslandMaps>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    islandroot_query: Query<Entity, With<IslandRoot>>
) {
    for (entity, island) in target_query.iter() {
        if let Some(map) = island_maps.get_map_mut(island.0) {
            if map.enemy_count == 0 {
                if let Ok(island_root) = islandroot_query.single() {
                    let top_tiles = map.above_water_top_tiles();
                    let chest_pos = top_tiles.choose(&mut rand::rng()).unwrap().clone() + IVec3::Y;

                    println!("{} spawning chest", chest_pos);
                    let chest_entity = commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.05, 1.05, 1.05))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 1.0, 1.0),
                            ..Default::default()
                        })),
                        Transform::from_xyz(chest_pos.x as f32, chest_pos.y as f32, chest_pos.z as f32),
                        Position(chest_pos),
                        Chest,
                        Health::new(30),
                        OnIsland(island.0),
                    )).insert(ChildOf(island_root)).id();

                    map.add_entity_ivec3(chest_pos, Tile::new(TileType::Enemy, chest_entity));
                    commands.entity(entity).insert(CompletedIslandObjective);
                }
            }
        }
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
    player_query: Query<(&Position, Entity, &OwnedBy, &OnIsland), With<Character>>,
    mut islands: ResMut<IslandMaps>,
    mut leave_island_event: EventWriter<ToClients<LeaveIsland>>,
) {
    for (position, entity, owner, island) in &player_query {
        if let Some(map) = islands.get_map_mut(island.0) {
            if position.0 == map.leave_position + IVec3::Y {
                println!("{:?} leaves island", entity);

                commands.entity(entity).despawn();
                map.remove_entity(position.0);
                map.player_count -= 1;
                leave_island_event.write(ToClients { mode: SendMode::Direct(owner.0), event: LeaveIsland(island.0) });    
            }
        }
    }
}

fn clean_up_island(
    mut commands: Commands,
    mut island_maps: ResMut<IslandMaps>,
    enemy_query: Query<(Entity, &OnIsland), With<Enemy>>,
    mut islands: Query<(Entity, &Island), With<MapFinishedIsland>>,
    players: Query<&OnIsland, Without<Enemy>>
) {
    let mut player_count: HashSet<u64> = HashSet::new();
    for island in players.iter() {
        player_count.insert(island.0);
    }

    island_maps.maps.retain(|id, _map| {
        if !player_count.contains(id) {
            println!("No players left on island {:?}: cleaning up", id);
            
            for (enemy_entity, island_id) in enemy_query.iter() {
                if *id == island_id.0 {
                    commands.entity(enemy_entity).despawn();

                }
            }
            for (entity, island_id) in islands.iter_mut() {
                if island_id.0 == *id {
                    commands.entity(entity).remove::<MapFinishedIsland>();
                }
            }
            false
        } else {
            true
        }
    });   
}