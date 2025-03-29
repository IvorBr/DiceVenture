use bevy::prelude::*;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::overworld::*;
use crate::components::island::*;
use crate::components::player::Player;
use crate::preludes::network_preludes::*;
use crate::IslandSet;
use crate::GameState;

pub struct IslandPlugin;
impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Island), setup_island)
        .add_systems(Update, (update_island, detect_objective.in_set(IslandSet)));
    }
}

//generate island, for now just a square
fn setup_island(
    island_info_query: Query<&IslandInfo, With<SelectedIsland>>,
    mut map_events: EventWriter<ToClients<MapUpdate>>,
    mut commands: Commands,
    mut map: ResMut<Map>,
) {
    for x in 0..16 {
        for z in 0..16 {            
            map_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: MapUpdate(UpdateType::LoadTerrain, IVec3::new(x, 0, z), 0),
            });
        }
    }

    let enemy_pos = IVec3::new(5,1,1);
    let enemy_id = commands.spawn((
            Enemy{..Default::default()},
            Position(enemy_pos),
            EleminationObjective
        )).id();

    println!("{:?}", enemy_id);

    map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));
}

fn detect_objective(
    target_query: Query<&EleminationObjective>
) {
    if target_query.iter().count() == 0 {
       // reward player
    }
}

fn detect_leave(
    player_query: Query<(&Position, Entity), With<Player>>
) {
    for (position, player_entity) in player_query.iter() {

    }
}

fn update_island(
    mut map_events: EventReader<MapUpdate>,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
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
                )).id();
                
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

