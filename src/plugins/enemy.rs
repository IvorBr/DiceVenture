use std::collections::BinaryHeap;
use std::collections::HashMap;
use bevy::prelude::*;

use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
use crate::objects::enemy::{Node, MoveTimer};

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(PreUpdate, init_enemy)
        .add_systems(Update, move_enemies);
    }
}

fn init_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemies: Query<(Entity, &Position), (With<Enemy>, Without<Transform>)>,
) {
    for (entity, position) in &enemies {
        commands.entity(entity).insert((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::srgb_u8(255, 100, 100)),
                transform: Transform::from_xyz(position.0.x as f32, position.0.y as f32, position.0.z as f32),
                ..Default::default()
            },
            MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating)))
        );
    }
}

fn move_enemies(
    time: Res<Time>,    
    mut enemies: Query<(&mut MoveTimer, &mut Position, Entity), (With<Enemy>, Without<Player>)>,
    players: Query<&Position, With<Player>>,
    mut map: ResMut<Map>
) {
    for (mut timer, mut enemy_pos, enemy_entity) in enemies.iter_mut() {
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
                let path = astar(enemy_pos.0, target_pos, &map);
                
                if let Some(next_step) = path.get(1) {
                    map.remove_entity(enemy_pos.0);
                    map.add_entity_ivec3(*next_step, Tile::new(TileType::Enemy, enemy_entity));

                    enemy_pos.0 = *next_step;
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