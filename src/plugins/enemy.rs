use bevy::prelude::*;

use crate::components::humanoid::{ActionState, AttackCooldowns};
use crate::components::island::OnIsland;
use crate::components::island_maps::{self, IslandMaps};
use crate::components::overworld::{LocalIsland, Island};
use crate::plugins::attack::{AttackCatalogue, AttackInfo, AttackRegistry, AttackSpec};
use crate::plugins::enemy_behaviour::AggressionPlugin;
use crate::plugins::enemy_movement::MovementPlugin;
use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::{Attacks, SnakePart};
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
            (init_enemy).in_set(IslandSet)
        )
        .add_systems(Update, (attack_check).run_if(server_running));
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

//TODO add system to easily add new attacks to enemies, probably at the enemy rules?
fn attack_check(
    mut commands: Commands,
    mut enemies: Query<(Entity, &Position, &mut AttackCooldowns, &Attacks), With<Enemy>>,
    players: Query<(Entity, &Position), With<Player>>,
    catalog: Res<AttackCatalogue>
) {
    for (enemy_entity, enemy_pos, mut cooldowns, attacks) in &mut enemies {
        // iterate over all attacks this enemy can use
        for id in &attacks.0 {
            if let Some(timer) = cooldowns.0.get_mut(&id) {
                if !timer.finished() { 
                    continue; 
                }
            }

            let spec = catalog.0.get(id).unwrap();

            if let Some((_, target_pos)) = players.iter().find(|(_, pos)| spec.offsets.contains(&(pos.0 - enemy_pos.0))) {
                let dir = target_pos.0 - enemy_pos.0;
            
                cooldowns.0.insert(*id, Timer::from_seconds(spec.cooldown, TimerMode::Once));

                commands.server_trigger_targets(
                    ToClients {
                        mode  : SendMode::Broadcast,
                        event : AttackInfo { attack_id: *id, offset: dir },
                    },
                    enemy_entity,
                );

                break;
            }
        }
    }
}