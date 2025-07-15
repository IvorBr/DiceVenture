use bevy::prelude::*;
use crate::attacks::base_attack::BaseAttack;
use crate::components::island::OnIsland;
use crate::components::island_maps::IslandMaps;
use crate::components::player::LocalPlayer;
use crate::components::player::MovementCooldown;
use crate::plugins::attack::key_of;
use crate::plugins::attack::AttackRegistry;
use crate::plugins::attack::ClientAttack;
use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;
use crate::plugins::camera::{DollyCamera, PlayerCamera};
use crate::IslandSet;
use super::network::OwnedBy;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_client_event::<MoveDirection>(Channel::Ordered)
        .insert_resource(MovementCooldown {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        })
        .add_systems(Update, (
            (apply_movement).run_if(server_running),
            (movement_input, attack_input).in_set(IslandSet)
        ));
    }
}

fn movement_input(
    mut move_events: EventWriter<MoveDirection>, 
    input: Res<ButtonInput<KeyCode>>,
    camera: Query<&DollyCamera, With<PlayerCamera>>,
    time: Res<Time>,
    mut cooldown: ResMut<MovementCooldown>,
) {
    cooldown.timer.tick(time.delta());

    let mut direction = IVec3::ZERO;
    let mut just_pressed = false;

    if input.just_pressed(KeyCode::KeyW) {
        direction.z -= 1;
        just_pressed = true;
    } else if input.just_pressed(KeyCode::KeyS) {
        direction.z += 1;
        just_pressed = true;
    } else if input.just_pressed(KeyCode::KeyD) {
        direction.x += 1;
        just_pressed = true;
    } else if input.just_pressed(KeyCode::KeyA) {
        direction.x -= 1;
        just_pressed = true;
    } else if cooldown.timer.finished() {
        if input.pressed(KeyCode::KeyW) {
            direction.z -= 1;
        } else if input.pressed(KeyCode::KeyS) {
            direction.z += 1;
        } else if input.pressed(KeyCode::KeyD) {
            direction.x += 1;
        } else if input.pressed(KeyCode::KeyA) {
            direction.x -= 1;
        }
    }

    if direction == IVec3::ZERO {
        return;
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

    if (just_pressed && cooldown.timer.elapsed_secs() >= 0.05)
        || (!just_pressed && cooldown.timer.finished())
    {
        move_events.send(MoveDirection(direction));
        cooldown.timer.reset();
    }
}

fn attack_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    camera: Query<&DollyCamera, With<PlayerCamera>>,
    player: Query<Entity, (With<LocalPlayer>, With<Player>)>,
    attack_reg: Res<AttackRegistry>,
) {
    let mut direction = IVec3::ZERO;

    if input.just_pressed(KeyCode::ArrowUp) {
        direction.z -= 1;
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        direction.z += 1;
    }
    if input.just_pressed(KeyCode::ArrowRight) {
        direction.x += 1;
    }
    if input.just_pressed(KeyCode::ArrowLeft) {
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
        let base_attack_key = key_of::<BaseAttack>();
        let entity = player.single();
        
        commands.client_trigger_targets(
            ClientAttack {
                attack_id: base_attack_key,
                offset: direction
            },
            entity
        );

        attack_reg.spawn(base_attack_key, &mut commands, entity, direction);
    }
}

fn apply_movement(
    mut move_events: EventReader<FromClient<MoveDirection>>,
    mut players: Query<(&OwnedBy, &mut Position, Entity, &OnIsland), With<Player>>,
    mut islands: ResMut<IslandMaps>,
) {
    for FromClient { client_entity, event } in move_events.read() {
        for (owner, mut position, player_entity, island) in &mut players {
            let map = islands.get_map_mut(island.0);

            if *client_entity == owner.0 {
                let mut new_position = event.0;
                let current_pos = position.0.clone();
                new_position.x += current_pos.x;
                new_position.y += current_pos.y;
                new_position.z += current_pos.z;
                
                match map.get_tile(new_position).kind {
                    TileType::Terrain(_) => {
                        new_position.y += 1;
                        let tile_above = map.get_tile(new_position);

                        if tile_above.kind != TileType::Empty {
                            return;
                        }
                    }
                    TileType::Empty => {
                        if new_position.y != 0 {
                            let mut temp_pos = new_position;
                            temp_pos.y -= 1;
                            let tile_below = map.get_tile(temp_pos);

                            if tile_below.kind == TileType::Empty {
                                temp_pos.y -= 1;
                                let tile_below_terrain = map.get_tile(temp_pos);
                                if !matches!(tile_below_terrain.kind, TileType::Terrain(_)) {
                                    return;
                                }
                                new_position.y -= 1;
                            }
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

