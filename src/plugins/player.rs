use bevy::prelude::*;

use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(PreUpdate, update_position)
        .add_systems(Update, (init_player, read_input, apply_movement));
    }
}

fn init_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Position), (With<Player>, Without<Transform>)>,
) {
    for (entity, position) in &players {
        commands.entity(entity).insert(
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::srgb_u8(255, 255, 255)),
                transform: Transform::from_xyz(position.0.x as f32, position.0.y as f32, position.0.z as f32),
                ..Default::default()
        });
    }
}

fn read_input(mut move_events: EventWriter<MoveDirection>, 
    input: Res<ButtonInput<KeyCode>>
) {
    let mut direction = IVec3::ZERO;

    if input.just_pressed(KeyCode::KeyW) {
        direction.z -= 1;
    }
    if input.just_pressed(KeyCode::KeyS) {
        direction.z += 1;
    }
    if input.just_pressed(KeyCode::KeyD) {
        direction.x += 1;
    }
    if input.just_pressed(KeyCode::KeyA) {
        direction.x -= 1;
    }
    if direction != IVec3::ZERO {
        move_events.send(MoveDirection(direction));
    }
}

fn apply_movement(
    mut move_events: EventReader<FromClient<MoveDirection>>,
    mut players: Query<(&Player, &mut Position, &Transform, Entity)>,
    mut enemies: Query<&mut Health, With<Enemy>>,
    mut map: ResMut<Map>
) {
    for FromClient { client_id, event } in move_events.read() {
        for (player, mut position, transform, player_entity) in &mut players {
            if *client_id == player.0 {
                let mut new_position = event.0;
                let current_pos = IVec3::new(transform.translation.x as i32, transform.translation.y as i32, transform.translation.z as i32);
                new_position.x += current_pos.x;
                new_position.y += current_pos.y;
                new_position.z += current_pos.z;
                
                match map.get_tile(new_position).kind {
                    TileType::Enemy => {
                        let tile = map.get_tile(new_position);
                        if let Ok(mut health) = enemies.get_mut(tile.entity) {
                            println!("Enemy encountered with health: {}", health.get());
                            health.damage(10);
                            println!("Enemy health after damage: {}", health.get());
                        }
                        return;
                    }
                    TileType::Terrain => {
                        new_position.y += 1;
                        let tile_above = map.get_tile(new_position);

                        if tile_above.kind != TileType::Empty {
                            return;
                        }
                    }
                    TileType::Empty => {
                        let mut temp_pos = new_position;
                        temp_pos.y -= 1;
                        let tile_below = map.get_tile(temp_pos);

                        if tile_below.kind == TileType::Empty {
                            temp_pos.y -= 1;
                            let tile_below_terrain = map.get_tile(temp_pos);
                            if tile_below_terrain.kind != TileType::Terrain {
                                return;
                            }
                            new_position.y -= 1;
                        }
                    }
                    _ => {
                        return;
                    }
                }
                
                // Update the map after the logic
                map.remove_entity(current_pos);
                map.add_entity_ivec3(new_position, Tile::new(TileType::Player, player_entity));

                position.0 = new_position;
            }
        }
    }
}

fn update_position(mut moved_players: Query<(&Position, &mut Transform), Changed<Position>>,
){
    for (position, mut transform) in &mut moved_players {
        transform.translation.x = position.0.x as f32;
        transform.translation.y = position.0.y as f32;
        transform.translation.z = position.0.z as f32;
    }
}