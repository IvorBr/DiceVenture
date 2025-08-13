use bevy::prelude::*;

use crate::components::humanoid::{ActionState, AttackCooldowns, VisualEntity, VisualRef};
use crate::components::island::OnIsland;
use crate::components::island_maps::IslandMaps;
use crate::components::overworld::{LocalIsland, Island};
use crate::plugins::attack::{AttackCatalogue, AttackInfo};
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
            ((init_enemy).in_set(IslandSet),
            (attack_check, enemy_death_check).run_if(server_running))
        );
    }
}

fn init_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemies: Query<(Entity, &Position, &OnIsland), (With<Enemy>, Without<Transform>)>,
    enemy_shapes: Query<&Shape>,
    snake_parts: Query<&SnakePart, Without<Transform>>,
    local_island_query: Query<&Island, With<LocalIsland>>,
) {
    for (entity, position, island) in &enemies {
        if let Ok(local_island) = local_island_query.single() {
            if island.0 != local_island.0 {
                continue;
            }
        }
        
        println!("{:?} enemy spawned", entity);

        let visual = commands.spawn((
            VisualEntity,
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(255, 255, 255),
                ..Default::default()
            })),
        ))
        .id();

        commands.entity(entity).add_child(visual).insert(VisualRef(visual));

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
    mut enemies: Query<(Entity, &Position, &mut AttackCooldowns, &Attacks, &ActionState), With<Enemy>>,
    players: Query<(Entity, &Position), With<Character>>,
    catalog: Res<AttackCatalogue>
) {
    for (enemy_entity, enemy_pos, mut cooldowns, attacks, action_state) in &mut enemies {
        // iterate over all attacks this enemy can use
        if *action_state != ActionState::Idle {
            continue;
        }

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

fn enemy_death_check(
    mut commands: Commands,
    entities: Query<(&OnIsland, &Health, Entity), With<Enemy>>,
    snake_parts: Query<&SnakePart>,
    mut island_maps: ResMut<IslandMaps>
) {
    for (island, health, entity) in &entities {
        if health.get() == 0 {
            island_maps.get_map_mut(island.0).map(|map| map.enemy_count -= 1);
            
            commands.entity(entity).insert(RemoveEntity);
            
            if let Ok(mut current) = snake_parts.get(entity) {
                while let Some(next_entity) = current.next {
                    commands.entity(next_entity).insert(RemoveEntity);
                    current = match snake_parts.get(next_entity) {
                        Ok(snake) => snake,
                        _ => break,
                    };
                }
            }
        }
    }
}
