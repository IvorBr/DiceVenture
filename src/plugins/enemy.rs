use bevy::prelude::*;

use crate::components::enemy::AttackPhase;
use crate::components::enemy::StartAttack;
use crate::components::enemy::WindUp;
use crate::components::humanoid::ActionState;
use crate::components::island::OnIsland;
use crate::components::overworld::{LocalIsland, Island};
use crate::plugins::enemy_aggression::AggressionPlugin;
use crate::plugins::enemy_movement::MovementPlugin;
use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::SnakePart;
use crate::IslandSet;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((MovementPlugin, AggressionPlugin))
        .replicate::<Enemy>()
        .replicate::<Shape>()
        .replicate::<SnakePart>()
        .add_systems(PreUpdate,
            (init_enemy, visual_windup_and_attack).in_set(IslandSet)
        )
        .add_systems(Update, ( 
            attack_clean_up,
            (attack_check).run_if(server_running)
        )
        )
        .add_server_trigger::<StartAttack>(Channel::Unordered)
        .add_observer(client_add_attack);
    }
}

fn init_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemies: Query<(Entity, &Position, &OnIsland), (With<Enemy>, Without<Transform>)>,
    enemy_shapes: Query<&Shape>,
    snake_parts: Query<&SnakePart, Without<Transform>>,
    local_island: Query<&Island, With<LocalIsland>>,
) {
    for (entity, position, island) in &enemies {
        if island.0 != local_island.single().0 {
            continue;
        }
        
        println!("{:?} enemy spawned", entity);

        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(200, 50, 50),
                ..Default::default()
            })),
            Transform::from_xyz(position.0.x as f32, position.0.y as f32, position.0.z as f32),
        ));

        if snake_parts.get(entity).is_ok() { //for now we just standardize a snake of size 5...
            let mut prev_entity = entity;
            for i in 1..5 {
                let offset_pos = position.0 - IVec3::new(i, 0, 0);
                let next_entity = commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb_u8(200, 50, 50),
                        ..Default::default()
                    })),
                    Transform::from_xyz(
                        offset_pos.x as f32,
                        offset_pos.y as f32,
                        offset_pos.z as f32,
                    ),
                    Position(offset_pos),
                )).id();

                println!("{}",next_entity);
                commands.entity(prev_entity).insert(
                    SnakePart {
                        next: Some(next_entity)
                    }
                );
                prev_entity = next_entity;
            }

            commands.entity(prev_entity).insert(
                SnakePart {
                    next: Some(Entity::PLACEHOLDER)
                }
            );
        }

        // Spawn visual parts for each offset
        if enemy_shapes.get(entity).is_ok() {
            for offset in &enemy_shapes.get(entity).expect("Shape was not found.").0 {
                let part_position = *offset;
                let child = commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb_u8(200, 50, 50),
                        ..Default::default()
                    })),
                    Transform::from_xyz(
                        part_position.x as f32,
                        part_position.y as f32,
                        part_position.z as f32,
                    )
                )).id();

                commands.entity(entity).add_child(child);
            }
        }
    }
}

fn client_add_attack(
    trigger: Trigger<StartAttack>,
    mut commands: Commands,
){
    commands.entity(trigger.entity()).insert(trigger.attack.clone());
}

fn attack_clean_up(
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut WindUp)>,
    time: Res<Time>,
){
    for (entity, mut windup) in enemies.iter_mut() {
        if windup.timer.finished() {
            if windup.phase == AttackPhase::Windup {
                windup.phase = AttackPhase::Strike;
                windup.timer = Timer::from_seconds(0.25, TimerMode::Once);
            }
            else {
                commands.entity(entity).remove::<WindUp>();
            }
        }

        windup.timer.tick(time.delta());
    }
}

//check if enemy can attack
fn attack_check(
    mut commands: Commands,
    enemies: Query<(Entity, &Position), (With<Enemy>, Without<Player>, Without<WindUp>)>,
    players: Query<&Position, With<Player>>,
) {
    for (enemy_entity, enemy_pos) in enemies.iter() {
        for player_pos in players.iter() {
            let delta = player_pos.0 - enemy_pos.0;

            if delta.abs().x + delta.abs().y + delta.abs().z == 1 {
                commands.server_trigger_targets(
                    ToClients {
                        mode: SendMode::Broadcast,
                        event: StartAttack {
                            attack: WindUp {
                                target_pos: player_pos.0,
                                timer: Timer::from_seconds(0.5, TimerMode::Once),
                                phase: AttackPhase::Windup
                            }
                        },
                    },
                    enemy_entity,
                );

                break;
            }
        }
    }
}

use std::f32::consts::PI;
fn visual_windup_and_attack(
    mut commands: Commands,
    mut enemies: Query<(Entity, &WindUp, &Position, &mut Transform, &mut ActionState), With<Enemy>>,
    players: Query<(Entity, &Position), With<Player>>,
) {

    for (entity, windup, enemy_pos, mut transform, mut action_state) in enemies.iter_mut() {
        match *action_state {
            ActionState::Idle => {
                commands.entity(entity).insert(ActionState::Attacking);
            }
            ActionState::Moving => continue,
            _ => {}
        }

        let direction = (windup.target_pos - enemy_pos.0).as_vec3().normalize_or_zero();
        let base_pos = enemy_pos.0.as_vec3();
        let forward_offset = direction * 0.4;

        // Phase-based tilt angle
        let angle = match windup.phase {
            AttackPhase::Windup => {
                let t = windup.timer.fraction();
                -30.0_f32.to_radians() * t
            }
            AttackPhase::Strike => {
                let t = windup.timer.fraction();
                (-30.0 + 50.0 * (PI * t).sin()).to_radians()
            }
            _ => 0.0
        };

        // Step 1: Face the direction
        let yaw = direction.x.atan2(direction.z); // +Z is forward
        let facing = Quat::from_rotation_y(-yaw); // Rotate around Y to face

        // Step 2: Tilt forward in local space
        let tilt = Quat::from_rotation_x(angle);

        // Step 3: Combine cleanly
        transform.rotation = facing * tilt;

        match windup.phase {
            AttackPhase::Windup => {
                transform.translation = base_pos;
            }
            AttackPhase::Strike => {
                let t = windup.timer.fraction();
                let eased = (PI * t).sin();
                transform.translation = base_pos + forward_offset * eased;

                if windup.timer.finished() {
                    transform.translation = base_pos;
                    transform.rotation = facing; // reset to facing direction

                    for (_, player_pos) in players.iter() {
                        if player_pos.0 == windup.target_pos {
                            println!("Player hit!");
                        }
                    }
                    *action_state = ActionState::Idle;
                }
            }
            _ => continue
        }
    }
}
