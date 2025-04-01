use bevy::prelude::*;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::overworld::*;
use crate::components::island::*;
use crate::components::player::LocalPlayer;
use crate::plugins::camera::NewCameraTarget;
use crate::components::player::Player;
use crate::preludes::network_preludes::*;
use crate::IslandSet;
use crate::GameState;

pub struct IslandPlugin;
impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Island), setup_island)
        .add_systems(OnExit(GameState::Island), island_cleanup)
        .add_systems(Update, (
            update_island, 
            (island_player_init, detect_leave, detect_objective).in_set(IslandSet)
        ));
    }
}

fn island_cleanup(
    mut commands: Commands,
    mut map: ResMut<Map>,
    players: Query<Entity, With<Player>>,
    islandroot_query: Query<Entity, With<IslandRoot>>
) {
    map.reset();

    //clean up entities
    if let Ok(island_root) = islandroot_query.get_single() {
        commands.entity(island_root).despawn_recursive();
    }

    for entity in players.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn island_player_init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Position, &Player), (With<Player>, Without<Transform>)>,
    client: Res<RepliconClient>,
) {
    let client_id = client.id();

    for (entity, position, player) in players.iter() {
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

        if (client_id.is_some() && player.0 == client_id.unwrap()) || (!client_id.is_some() && player.0 == ClientId::SERVER) {
            commands.entity(entity).insert(NewCameraTarget);
            commands.entity(entity).insert(LocalPlayer);
        }
    }
}

//generate island, for now just a square
fn setup_island(
    island_info_query: Query<&IslandInfo, With<SelectedIsland>>,
    mut map_events: EventWriter<ToClients<MapUpdate>>,
    mut commands: Commands,
    mut map: ResMut<Map>,
    client: Res<RepliconClient>,
) {
    let island_root = commands
        .spawn((
            IslandRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            InheritedVisibility::VISIBLE
        )).id();

    for x in 0..16 {
        for z in 0..16 {            
            map_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: MapUpdate(UpdateType::LoadTerrain, IVec3::new(x, 0, z), 0),
            });
        }
    }

    //setup leave tile
    map_events.send(ToClients {
        mode: SendMode::Broadcast,
        event: MapUpdate(UpdateType::LoadTerrain, IVec3::new(8, 0, 16), 0),
    });
    
    //add one enemy
    let enemy_pos = IVec3::new(5,1,1);
    let enemy_id = commands.spawn((
            Enemy{..Default::default()},
            Position(enemy_pos),
            EleminationObjective
        )).set_parent(island_root)
        .id();

    map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));

    //spawn player
    let client_id = client.id();
    if client_id.is_some() {
        commands.spawn((
            Player(client_id.unwrap()),
            Position(IVec3::new(6,1,5))
        ));
    } else {
        commands.spawn((
            Player(ClientId::SERVER), 
            Position(IVec3::new(6,1,5))
        ));
    }
}

fn detect_objective(
    target_query: Query<&EleminationObjective>
) {
    if target_query.iter().count() == 0 {
       // reward player
    }
}

fn detect_leave(
    player_query: Query<(&Position, Entity), With<Player>>,
    mut state: ResMut<NextState<GameState>>
) {
    for (position, player_entity) in player_query.iter() {
        if position.0 == IVec3::new(8, 1, 16) {
            println!("{:?} leaves island", player_entity);
            state.set(GameState::Overworld);
        }
    }
}

fn update_island(
    mut map_events: EventReader<MapUpdate>,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    islandroot_query: Query<Entity, With<IslandRoot>>
) {
    if let Ok(island_root) = islandroot_query.get_single() {
        for event in map_events.read() {
            match event.0 {
                UpdateType::LoadTerrain => {
                    let terrain_id = commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb_u8(100, 255, 100),
                            ..Default::default()
                        })),
                        Transform::from_xyz(event.1.x as f32, 0.0, event.1.z as f32)
                    )).set_parent(island_root)
                    .id();
                    
                    //event.3 has the tile type, currently hard coded...
                    map.add_entity_ivec3(event.1, Tile::new(TileType::Terrain, terrain_id));
                }
                UpdateType::UnloadTerrain => {
                    if let Some(chunk) = map.chunks.get(&event.1) {
                        let mut entities_to_despawn = Vec::new();
        
                        for tile in &chunk.tiles {
                            if commands.get_entity(tile.entity).is_some() && tile.kind == TileType::Terrain { //BIG PROBLEM CURRENTLY WITH ENEMY REPLICATION!!! check trello
                                entities_to_despawn.push(tile.entity);
                            }
                        }
                    
                        for entity in entities_to_despawn {
                            commands.entity(entity).despawn();
                        }
                        
                        map.chunks.remove(&event.1);
                    }
                }
            }   
        }
    }
}

