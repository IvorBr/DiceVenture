use bevy::prelude::*;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use crate::components::enemy::MoveRule;
use crate::components::enemy::EnemyState;
use crate::components::humanoid::ActionState;
use crate::components::island::OnIsland;
use crate::components::island_maps::IslandMaps;
use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::{PathfindNode, MoveTimer};

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                standard_mover,
            )
        );
    }
}

fn standard_mover(
    time: Res<Time>,
    mut enemies: Query<(Entity, &mut MoveTimer, &mut Position, &OnIsland, &EnemyState, &MoveRule, &ActionState), (With<Enemy>, Without<Player>)>,
    players: Query<&Position, With<Player>>,
    mut islands: ResMut<IslandMaps>,
) {
    for (enemy_entity, mut timer, mut enemy_pos, island, enemy_state, move_rule, action_state) in enemies.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            timer.1 = true;
        }

        if *action_state != ActionState::Idle || !timer.1 {
            continue;
        }

        if let Some(map) = islands.maps.get_mut(&island.0) {
            let enemy_target = match enemy_state {
                EnemyState::Attacking(target) => Some(players.get(*target)),
                _ => None,
            };

            if let Some(Ok(target_pos)) = enemy_target {
                let closest_offset = enemy_pos.0;
                let path = astar(closest_offset, target_pos.0, &map, move_rule);

                if let Some(next_step) = path.get(1) {
                    map.remove_entity(enemy_pos.0);
                    enemy_pos.0 = *next_step + (enemy_pos.0 - closest_offset);
                    map.add_entity_ivec3(enemy_pos.0, Tile::new(TileType::Enemy, enemy_entity));
                    
                    timer.0.reset();
                    timer.1 = false;
                }
            }
        }
    }
}


fn astar(start: IVec3, goal: IVec3, map: &Map, move_rule: &MoveRule) -> Vec<IVec3> {
    let mut open_set = BinaryHeap::new();
    let mut open_set_hash = HashSet::new(); // Store the positions in the open set for faster checks
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();
    let mut scores = HashMap::new();

    let start_heuristic = (move_rule.heuristic)(start, goal);
    open_set.push(PathfindNode { pos: start, f_score: start_heuristic });
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

        for neighbor in get_valid_neighbors(current, map, move_rule.offsets) {
            if closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g_score = current_g_score + 1;
            let (neighbor_g_score, _) = scores.get(&neighbor).unwrap_or(&(i32::MAX, i32::MAX));

            if tentative_g_score < *neighbor_g_score {
                came_from.insert(neighbor, current);
                let neighbor_f_score = tentative_g_score + (move_rule.heuristic)(neighbor, goal);
                scores.insert(neighbor, (tentative_g_score, neighbor_f_score));

                if !open_set_hash.contains(&neighbor) {
                    open_set.push(PathfindNode {
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

fn reconstruct_path(came_from: HashMap<IVec3, IVec3>, mut current: IVec3) -> Vec<IVec3> {
    let mut total_path = vec![]; // TODO: Include the goal node?
    while let Some(&prev) = came_from.get(&current) {
        current = prev;
        total_path.push(current);
    }
    total_path.reverse();
    total_path
}

fn get_valid_neighbors(position: IVec3, map: &Map, directions: &'static [IVec3]) -> Vec<IVec3> {
    let mut neighbors = Vec::new();

    for &dir in directions.iter() {
        let target = position + dir;
        if map.can_move(target) {
            neighbors.push(target);
            continue;
        }

        let climb = target + IVec3::Y;
        if map.can_move(climb) {
            neighbors.push(climb);
            continue;
        }

        let drop = target - IVec3::Y;
        if map.can_move(drop) {
            neighbors.push(drop);
            continue;
        }
    }

    neighbors
}

// fn move_enemies(
//     time: Res<Time>,    
//     mut enemies: Query<(Option<&SnakePart>, &mut MoveTimer, &mut Position, Entity, Option<&Shape>, &OnIsland, &EnemyState), (With<Enemy>, Without<Player>, Without<WindUp>)>,
//     players: Query<&Position, With<Player>>,
//     mut islands: ResMut<IslandMaps>,
//     mut snake_parts: Query<(&SnakePart, &mut Position), (Without<Enemy>, Without<Player>)>,
// ) {
//     for (snake_part, mut timer, mut enemy_pos, enemy_entity, shape, island, enemy_state) in enemies.iter_mut() {
//         if let Some(map) = islands.maps.get_mut(&island.0) {
//             if timer.0.tick(time.delta()).just_finished() {
                
//                 let enemy_target = match enemy_state {
//                     EnemyState::Attacking(target) => Some(players.get(*target).unwrap().0),
//                     _ => None
//                 };

//                 if let Some(target_pos) = enemy_target {
//                     let mut closest_offset = enemy_pos.0;
//                     let mut min_distance = closest_offset.distance_squared(target_pos);
                    
//                     //TODO: shape enemies should just have a main part
//                     if let Some(shape) = shape {
//                         for offset in &shape.0 {
//                             let offset_pos = enemy_pos.0 + *offset;
//                             let distance = offset_pos.distance_squared(target_pos);
        
//                             if distance < min_distance {
//                                 min_distance = distance;
//                                 closest_offset = offset_pos;
//                             }
//                         }
//                     }
                    
//                     let path = astar(closest_offset, target_pos, &map);
                    
//                     if let Some(next_step) = path.get(1) {
//                         map.remove_entity(enemy_pos.0);
                        
//                         if let Some(shape) = shape {
//                             for offset in &shape.0 {
//                                 let current_tile_pos = enemy_pos.0 + *offset;
//                                 map.remove_entity(current_tile_pos);
//                             }
//                         }
                        
//                         //TODO: snake movement logic
//                         if let Some(head) = snake_part {
//                             if let Some(next_entity) = head.next {
//                                 if let Ok(mut current) = snake_parts.get_mut(next_entity) {
//                                     let mut old_pos = current.1.0;
//                                     map.remove_entity(current.1.0);
//                                     current.1.0 = enemy_pos.0;
//                                     map.add_entity_ivec3(current.1.0, Tile::new(TileType::Enemy, enemy_entity));
                        
//                                     while let Some(next_entity) = current.0.next {
//                                         if let Ok(mut snake) = snake_parts.get_mut(next_entity) {
//                                             let next_old_pos = snake.1.0;
//                                             map.remove_entity(snake.1.0);
//                                             snake.1.0 = old_pos;
//                                             map.add_entity_ivec3(snake.1.0, Tile::new(TileType::Enemy, enemy_entity));
//                                             old_pos = next_old_pos;
//                                             current = snake;
//                                         } else {
//                                             break;
//                                         }
//                                     }
//                                 }
//                             }
//                         }
    
//                         enemy_pos.0 = *next_step + (enemy_pos.0 - closest_offset);
//                         map.add_entity_ivec3(enemy_pos.0, Tile::new(TileType::Enemy, enemy_entity));
    
//                         if let Some(shape) = shape {
//                             for offset in &shape.0 {
//                                 let new_tile_pos = enemy_pos.0 + *offset;
//                                 map.add_entity_ivec3(new_tile_pos, Tile::new(TileType::Enemy, enemy_entity));
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
