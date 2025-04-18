use bevy::prelude::*;

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

//generate island, for now just a square
fn client_setup_island(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let island_root = commands
        .spawn((
            IslandRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            InheritedVisibility::VISIBLE
        )).id();

    for x in 0..16 {
        for z in 0..16 {            
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(100, 255, 100),
                    ..Default::default()
                })),
                Transform::from_xyz(x as f32, 0.0, z as f32)
            )).set_parent(island_root);
        }
    }

    //setup leave tile
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(100, 255, 100),
            ..Default::default()
        })),
        Transform::from_xyz(8.0, 0.0, 16.0)
    )).set_parent(island_root);
}

fn server_setup_island(commands: &mut Commands,map: &mut Map, island: u64){
    //TODO: CURRENTLY HARDCODED!!! VERY BAD!!!
    //setup island tiles
    for x in 0..16 {
        for z in 0..16 {                        
            map.add_entity_ivec3(IVec3::new(x, 0, z), Tile::new(TileType::Terrain, Entity::PLACEHOLDER));
        }
    }

    //setup leave tiles
    map.add_entity_ivec3(IVec3::new(8, 0, 16), Tile::new(TileType::Terrain, Entity::PLACEHOLDER));

    let enemy_pos = IVec3::new(5,1,1);
    let enemy_id = commands.spawn((
            Enemy{..Default::default()},
            Position(enemy_pos),
            EleminationObjective,
            MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating)),
            OnIsland(island)
        )).id();
        
    map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));
}

fn player_enters_island(
    mut commands: Commands,
    mut island_enter_event: EventReader<FromClient<EnteredIsland>>,
    mut islands: ResMut<IslandMaps>,
) {
    for FromClient { client_entity, event } in island_enter_event.read() {
        let mut spawn_pos = IVec3::new(6, 1, 5);
        let island_id = event.0;

        let map = islands.maps.entry(island_id).or_insert_with(|| {
            let mut new_map = Map::new();
            server_setup_island(&mut commands, &mut new_map, island_id);
            new_map
        });

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
    if target_query.iter().count() == 0 {
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
        if position.0 == IVec3::new(8, 1, 16) {
            let map = islands.get_map_mut(island.0);
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

