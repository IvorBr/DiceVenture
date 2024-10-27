use bevy::prelude::*;

mod objects;
mod preludes;
mod plugins;
mod constants;

use crate::preludes::network_preludes::*;
use plugins::network::NetworkPlugin;
use plugins::enemy::EnemyPlugin;
use plugins::player::PlayerPlugin;
use plugins::camera::CameraPlugin;
use plugins::humanoid::HumanoidPlugin;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins, 
        NetworkPlugin, 
        PlayerPlugin,
        CameraPlugin,
        EnemyPlugin,
        HumanoidPlugin
    ))
    .add_systems(PreUpdate, update_map.after(ClientSet::Receive))
    .run();
}

fn update_map(mut map_events: EventReader<MapUpdate>,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    for event in map_events.read() {
        match event.0 {
            UpdateType::LoadTerrain => {
                let terrain_id = commands.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: materials.add(Color::srgb_u8(100, 255, 100)),
                    transform: Transform::from_xyz(event.1.x as f32, 0.0, event.1.z as f32),
                    ..Default::default()
                }).id();
                
                //event.3 has the tile type, currently hard coded...
                map.add_entity_ivec3(event.1, Tile::new(TileType::Terrain, terrain_id));
            }
            UpdateType::UnloadTerrain => {
                if let Some(chunk) = map.chunks.get(&event.1) {
                    let mut entities_to_despawn = Vec::new();
    
                    for tile in &chunk.tiles {
                        if commands.get_entity(tile.entity).is_some() && tile.kind == TileType::Terrain { //BIG PROBLEM CURRENTLY WITH ENEMY REPLICATION!!!
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