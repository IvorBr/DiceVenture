use bevy::prelude::*;

use dice_venture::AppPlugin;
use dice_venture::preludes::network_preludes::*;
use dice_venture::preludes::humanoid_preludes::*;
use dice_venture::objects::enemy::SnakePart;

fn main() {
    App::new()
    .add_plugins(AppPlugin)
    .add_systems(PreUpdate, update_map.after(ClientSet::Receive))
    .add_systems(Update, test_function)
    .run();
}

fn test_function(
    mut commands: Commands,
    mut map: ResMut<Map>,
    input: Res<ButtonInput<KeyCode>>,
){
    if input.just_pressed(KeyCode::KeyT) {
        //test multi_tile enemy
        let enemy_shape = Shape::new_2x2x2();
        let offsets = enemy_shape.0.clone();
        let enemy_pos = IVec3::new(1, 1, 0);

        let enemy_id = commands.spawn((EnemyBundle {
                health: Health::new(100),
                position: Position(enemy_pos),
                replicated: Replicated,
                enemy: Enemy
            },
            enemy_shape)).id();

        map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));
        
        for pos in offsets {
            let offset_pos = pos + enemy_pos;
            map.add_entity_ivec3(offset_pos, Tile::new(TileType::Enemy, enemy_id));
        }
    }
    else if input.just_pressed(KeyCode::KeyF) {
        //single tile snake
        println!("Snake");
        let enemy_pos = IVec3::new(1, 1, 0);
        let enemy_id = commands.spawn((EnemyBundle {
                health: Health::new(100),
                position: Position(enemy_pos),
                replicated: Replicated,
                enemy: Enemy
            },
            SnakePart {
                next: Some(Entity::PLACEHOLDER),
            }
        )).id();
            
        map.add_entity_ivec3(enemy_pos, Tile::new(TileType::Enemy, enemy_id));
        
        for i in 0..5 {
            let offset_pos = enemy_pos - IVec3::new(i, 0, 0);
            map.add_entity_ivec3(offset_pos, Tile::new(TileType::Enemy, enemy_id));
        }
    }
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