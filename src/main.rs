use bevy::prelude::*;

use dice_venture::AppPlugin;
use dice_venture::preludes::network_preludes::*;
use dice_venture::preludes::humanoid_preludes::*;
use dice_venture::components::enemy::{SnakePart, MovementType};

fn main() {
    App::new()
    .add_plugins(AppPlugin)
    // .add_systems(PreUpdate, update_map.after(ClientSet::Receive))
    // .add_systems(Update, test_function)
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
                enemy: Enemy { movement : MovementType::Multi }
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
                enemy: Enemy { movement: MovementType::Snake }
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