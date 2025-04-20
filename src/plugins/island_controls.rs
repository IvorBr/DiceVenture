use bevy::prelude::*;
use crate::components::humanoid::ActionState;
use crate::components::humanoid::AttackAnimation;
use crate::components::humanoid::AttackDirection;
use crate::components::humanoid::AttackLerp;
use crate::components::island::OnIsland;
use crate::components::island_maps::IslandMaps;
use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;
use crate::plugins::camera::{DollyCamera, PlayerCamera};
use crate::IslandSet;

use super::network::OwnedBy;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (
            (apply_attack, apply_movement).run_if(server_running),
            (movement_input, attack_input, animate_attack, update_attack_lerp).in_set(IslandSet)
        ));
    }
}

fn movement_input(
    mut move_events: EventWriter<MoveDirection>, 
    input: Res<ButtonInput<KeyCode>>,
    camera: Query<&DollyCamera,  With<PlayerCamera>>,
) {
    let mut direction = IVec3::ZERO;

    if input.pressed(KeyCode::KeyW) {
        direction.z -= 1;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.z += 1;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1;
    }
    if input.pressed(KeyCode::KeyA) {
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

fn attack_input(
    mut attack_events: EventWriter<AttackDirection>,
    input: Res<ButtonInput<KeyCode>>,
    camera: Query<&DollyCamera, With<PlayerCamera>>,
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
        attack_events.send(AttackDirection(direction));
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
                    // TileType::Enemy => {
                    //     let tile = map.get_tile(new_position);
                    //     if let Ok(mut health) = enemies.get_mut(tile.entity) {
                    //         println!("Enemy encountered with health: {}", health.get());
                    //         health.damage(10);
                    //         println!("Enemy health after damage: {}", health.get());
                    //     }
                    //     return;
                    // }
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

fn apply_attack(
    mut attack_events: EventReader<FromClient<AttackDirection>>,
    mut attack_animation_events: EventWriter<ToClients<AttackAnimation>>,
    players: Query<(&OwnedBy, &Position, &OnIsland), With<Player>>,
    mut enemies: Query<&mut Health, With<Enemy>>,
    islands: Res<IslandMaps>,
) {
    for FromClient { client_entity, event } in attack_events.read() {
        for (owner, position, island) in &players {
            if *client_entity != owner.0 {
                continue;
            }

            let map = islands.get_map(island.0);

            attack_animation_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: AttackAnimation {
                    client_entity: *client_entity,
                    direction: event.0
                }
            });

            let target_pos = position.0 + event.0;
            match map.get_tile(target_pos).kind {
                TileType::Enemy => {
                    let tile = map.get_tile(target_pos);
                    if let Ok(mut health) = enemies.get_mut(tile.entity) {
                        println!("Enemy encountered with health: {}", health.get());
                        health.damage(10);
                        println!("Enemy health after damage: {}", health.get());
                    }
                }
                _ => {
                }
            }
        }
    }
}

fn animate_attack(
    mut commands: Commands,
    mut attack_animation_events: EventReader<AttackAnimation>,
    mut players: Query<(Entity, &OwnedBy, &mut ActionState), With<Player>>
) {
    for AttackAnimation { client_entity, direction } in attack_animation_events.read() {
        for (entity, owner, mut action_state) in players.iter_mut() {
            if *client_entity != owner.0 {
                continue;
            }
            
            *action_state = ActionState::Attacking;
            commands.entity(entity).insert(AttackLerp {
                direction: *direction,
                timer: Timer::from_seconds(0.1, TimerMode::Once)
            });
        }
    }
}

fn update_attack_lerp(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &mut Transform, &mut AttackLerp, &mut ActionState)>,
) {
    for (entity, position, mut transform, mut lerp, mut action_state) in &mut query {
        lerp.timer.tick(time.delta());

        let t = lerp.timer.elapsed_secs() / lerp.timer.duration().as_secs_f32();
        let t = t.clamp(0.0, 1.0);

        let offset = if t < 0.5 {
            lerp.direction.as_vec3() * (t * 5.0 * 0.2)
        } else {
            lerp.direction.as_vec3() * ((1.0 - t) * 5.0 * 0.2)
        };

        let base = position.0.as_vec3();
        transform.translation = base + offset;

        if lerp.timer.finished() {
            transform.translation = base;
            commands.entity(entity).remove::<AttackLerp>();
            *action_state = ActionState::Idle;
        }
    }
}
