use std::collections::BinaryHeap;
use std::collections::HashMap;
use bevy::prelude::*;

use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
use crate::objects::enemy::{Node, MoveTimer, SnakePart};

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(PreUpdate, init_enemy)
        .add_systems(Update, move_enemies.run_if(server_running));
    }
}

fn init_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemies: Query<(Entity, &Position), (With<Enemy>, Without<Transform>)>,
    enemy_shapes: Query<&Shape>,
    snake_parts: Query<&SnakePart, Without<Transform>>,
) {
    for (entity, position) in &enemies {
        commands.entity(entity).insert((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::srgb_u8(200, 50, 50)),
                transform: Transform::from_xyz(position.0.x as f32, position.0.y as f32, position.0.z as f32),
                ..Default::default()
            },
            MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating)))
        );

        if snake_parts.get(entity).is_ok() { //for now we just standardize a snake of size 5...
            let mut prev_entity = entity;
            for i in 1..5 {
                let offset_pos = position.0 - IVec3::new(i, 0, 0);
                let next_entity = commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                        material: materials.add(Color::srgb_u8(200, 50, 50)),
                        transform: Transform::from_xyz(offset_pos.x as f32, offset_pos.y as f32, offset_pos.z as f32),
                        ..Default::default()
                    },
                    Position(offset_pos))
                ).id();
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
            for offset in &enemy_shapes.get(entity).unwrap().0 {
                let part_position = *offset;
                let child = commands.spawn(
                    PbrBundle {
                        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                        material: materials.add(Color::srgb_u8(200, 50, 50)),
                        transform: Transform::from_xyz(
                            part_position.x as f32,
                            part_position.y as f32,
                            part_position.z as f32,
                        ),
                        ..Default::default()
                    }).id();

                commands.entity(entity).add_child(child);
            }
        }
    }
}

fn move_enemies(
    time: Res<Time>,    
    mut enemies: Query<(Option<&SnakePart>, &mut MoveTimer, &mut Position, Entity, Option<&Shape>), (With<Enemy>, Without<Player>)>,
    players: Query<&Position, With<Player>>,
    mut map: ResMut<Map>,
    mut snake_parts: Query<(&SnakePart, &mut Position), (Without<Enemy>, Without<Player>)>,
) {
    for (snake_part, mut timer, mut enemy_pos, enemy_entity, shape) in enemies.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            let mut closest_player: Option<IVec3> = None;
            let mut closest_distance: i32 = i32::MAX;

            for player_pos in players.iter() {
                let cur_distance = enemy_pos.0.distance_squared(player_pos.0);
                if cur_distance < closest_distance {
                    closest_distance = cur_distance;
                    closest_player = Some(player_pos.0);
                }
            }

            if let Some(target_pos) = closest_player {
                let mut closest_offset = enemy_pos.0;
                let mut min_distance = closest_offset.distance_squared(target_pos);
                
                if let Some(shape) = shape {
                    for offset in &shape.0 {
                        let offset_pos = enemy_pos.0 + *offset;
                        let distance = offset_pos.distance_squared(target_pos);
    
                        if distance < min_distance {
                            min_distance = distance;
                            closest_offset = offset_pos;
                        }
                    }
                }
                
                let path = astar(closest_offset, target_pos, &map);
                
                if let Some(next_step) = path.get(1) {
                    map.remove_entity(enemy_pos.0);
                    
                    if let Some(shape) = shape {
                        for offset in &shape.0 {
                            let current_tile_pos = enemy_pos.0 + *offset;
                            map.remove_entity(current_tile_pos);
                        }
                    }
                    
                    if let Some(head) = snake_part {
                        if let Some(next_entity) = head.next {
                            if let Ok(mut current) = snake_parts.get_mut(next_entity) {
                                let mut old_pos = current.1.0;
                                map.remove_entity(current.1.0);
                                current.1.0 = enemy_pos.0;
                                map.add_entity_ivec3(current.1.0, Tile::new(TileType::Enemy, enemy_entity));
                    
                                while let Some(next_entity) = current.0.next {
                                    if let Ok(mut snake) = snake_parts.get_mut(next_entity) {
                                        let next_old_pos = snake.1.0;
                                        map.remove_entity(snake.1.0);
                                        snake.1.0 = old_pos;
                                        map.add_entity_ivec3(snake.1.0, Tile::new(TileType::Enemy, enemy_entity));
                                        old_pos = next_old_pos;
                                        current = snake;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    enemy_pos.0 = *next_step + (enemy_pos.0 - closest_offset);
                    map.add_entity_ivec3(enemy_pos.0, Tile::new(TileType::Enemy, enemy_entity));

                    if let Some(shape) = shape {
                        for offset in &shape.0 {
                            let new_tile_pos = enemy_pos.0 + *offset;
                            map.add_entity_ivec3(new_tile_pos, Tile::new(TileType::Enemy, enemy_entity));
                        }
                    }
                }
            }
        }
    }
}

fn astar(start: IVec3, goal: IVec3, map: &Map) -> Vec<IVec3> {
    let mut open_set = BinaryHeap::new();
    let mut open_set_hash = HashSet::new(); // Store the positions in the open set for faster checks
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();
    
    // Combine g_score and f_score into a single HashMap of tuples (g, f)
    let mut scores = HashMap::new();

    let start_heuristic = heuristic(start, goal);
    open_set.push(Node { pos: start, f_score: start_heuristic });
    open_set_hash.insert(start);
    scores.insert(start, (0, start_heuristic));

    while let Some(current_node) = open_set.pop() {
        let current = current_node.pos;

        if current == goal {
            return reconstruct_path(came_from, current);
        }

        open_set_hash.remove(&current);
        closed_set.insert(current);

        let (current_g_score, _) = scores[&current];

        for neighbor in get_valid_neighbors(current, map) {
            if closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g_score = current_g_score + 1;

            // Fetch the neighbor's g_score if it exists, otherwise assume a large value
            let (neighbor_g_score, _) = scores.get(&neighbor).unwrap_or(&(i32::MAX, i32::MAX));

            if tentative_g_score < *neighbor_g_score {
                came_from.insert(neighbor, current);
                let neighbor_f_score = tentative_g_score + heuristic(neighbor, goal);
                scores.insert(neighbor, (tentative_g_score, neighbor_f_score));

                // Only push to open set if it's not already there
                if !open_set_hash.contains(&neighbor) {
                    open_set.push(Node {
                        pos: neighbor,
                        f_score: neighbor_f_score,
                    });
                    open_set_hash.insert(neighbor);
                }
            }
        }
    }

    Vec::new()
}

// Heuristic function for A*
fn heuristic(a: IVec3, b: IVec3) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
}

// Reconstruct the path
fn reconstruct_path(came_from: HashMap<IVec3, IVec3>, mut current: IVec3) -> Vec<IVec3> {
    let mut total_path = vec![]; // Include the goal node
    while let Some(&prev) = came_from.get(&current) {
        current = prev;
        total_path.push(current);
    }
    total_path.reverse(); // Path needs to be reversed because we build it backward
    total_path
}

fn get_valid_neighbors(position: IVec3, map: &Map) -> Vec<IVec3> {
    let mut neighbors = Vec::new();

    let directions = [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ];

    for &dir in directions.iter() {
        let neighbor_pos = position + dir;
        if map.can_move(neighbor_pos) {
            neighbors.push(neighbor_pos);
        }
    }

    neighbors
}