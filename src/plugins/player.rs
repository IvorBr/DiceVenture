use bevy::prelude::*;

use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;
use crate::components::player::LocalPlayer;
use crate::plugins::camera::{DollyCamera, PlayerCamera};

use super::camera::CameraTarget;
use crate::IslandSet;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (init_player, read_input, apply_movement).in_set(IslandSet));
    }
}

fn init_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Position, &Player), (With<Player>, Without<Transform>)>,
    client: Res<RepliconClient>,
) {
    let client_id = client.id();

    for (entity, position, player) in &players {
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
            commands.entity(entity).insert(LocalPlayer);
            commands.entity(entity).insert(CameraTarget);
        }
    }
}

fn read_input(mut move_events: EventWriter<MoveDirection>, 
    input: Res<ButtonInput<KeyCode>>,
    camera: Query<&DollyCamera,  With<PlayerCamera>>
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

    if let Ok(camera) = camera.get_single() {
        direction = match camera.direction {
            0 => direction,                    
            1 => IVec3::new(direction.z, 0, -direction.x),
            2 => IVec3::new(-direction.x, 0, -direction.z),
            3 => IVec3::new(-direction.z, 0, direction.x),
            _ => direction,
        };
    }

    if direction != IVec3::ZERO {
        move_events.send(MoveDirection(direction));
    }
}

fn apply_movement(
    mut move_events: EventReader<FromClient<MoveDirection>>,
    mut players: Query<(&Player, &mut Position, Entity), With<Player>>,
    mut enemies: Query<&mut Health, With<Enemy>>,
    mut map: ResMut<Map>
) {
    for FromClient { client_id, event } in move_events.read() {
        for (player, mut position, player_entity) in &mut players {
            if *client_id == player.0 {
                let mut new_position = event.0;
                let current_pos = position.0.clone();
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