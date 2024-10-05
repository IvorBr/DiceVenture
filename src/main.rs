use bevy::prelude::*;
use std::collections::HashMap;

pub mod game_objects;

pub mod preludes;
use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;

pub mod plugins;
use plugins::network_plugin::NetworkPlugin;

#[derive(Component)]
struct MyCameraMarker;

#[derive(Component)]
struct MoveTimer(Timer);

fn main() {
    App::new()
    .add_plugins((DefaultPlugins, NetworkPlugin))
    .add_systems(Startup, setup)
    .add_systems(PreUpdate,
        (
            death_check.run_if(server_running).before(remove_entities),
            (init_player, init_enemy, remove_entities).after(ClientSet::Receive),
            update_map
                .after(ServerSet::Receive)
                .run_if(client_connected),
            update_position,
        )
    )
    .add_systems(Update, (read_input, apply_movement, move_enemies))
    .run();
}

fn update_map(mut map_events: EventReader<MapUpdate>,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    for event in map_events.read() {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(100, 255, 100)),
            transform: Transform::from_xyz(event.0.x as f32, 0.0, event.0.z as f32),
            ..Default::default()
        });
        
        map.add_entity_ivec3(event.0, event.2);
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
            MoveTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        );
    }
}

fn death_check(
    mut commands: Commands,
    entities: Query<(&Health, Entity), Or<(With<Player>, With<Enemy>)>>
) {
    for (health, entity) in &entities {
        if health.get() == 0 {
            println!("{}, {}", entity, health.get());
            commands.entity(entity).insert(RemoveEntity);
        }
    }
}

fn remove_entities(mut commands: Commands,
    entities: Query<(Entity, &Position), With<RemoveEntity>>,
    mut map: ResMut<Map>
) {
    for (entity, position) in &entities {
        map.remove_entity(position.0);
        println!("Despawning entity: {:?}", entity);
        commands.entity(entity).despawn();
    }
}

fn setup(mut commands: Commands
) {
    commands.spawn((
        MyCameraMarker,
        Camera3dBundle {
            projection: PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                ..default()
            }.into(),
            transform: Transform::from_xyz(0.0, 10.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
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

                match map.cell(new_position) {
                    Some(Tile::Enemy(enemy_entity)) => {
                        if let Ok(mut health) = enemies.get_mut(enemy_entity) {
                            println!("Enemy encountered with health: {}", health.get());
                            health.damage(10);
                            println!("Enemy health after damage: {}", health.get());
                        }
                        return;
                    }
                    Some(Tile::Terrain) => {
                        new_position.y += 1;
                        if map.cell(new_position) != None || map.cell(new_position) != None {
                            return;
                        }
                    }
                    None => {
                        let mut temp_pos = new_position;
                        temp_pos.y -= 1;
                        if map.cell(temp_pos) == None {
                            temp_pos.y -= 1;
                            if map.cell(temp_pos) != Some(Tile::Terrain){
                                return;
                            }
                            new_position.y -= 1;
                        }
                    }
                    _ => {
                        return;
                    }
                }
                
                map.remove_entity(current_pos);
                map.add_entity_ivec3(new_position, Tile::Player(player_entity));

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

fn move_enemies(
    time: Res<Time>,    
    mut enemies: Query<(&mut MoveTimer, &mut Transform, &mut Position, Entity), (With<Enemy>, Without<Player>)>,
    players: Query<&Position, With<Player>>,
    mut map: ResMut<Map>
) {
    for (mut timer, mut transform, mut enemy_pos, enemy_entity) in enemies.iter_mut() {
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
                
                println!("new path from: {:?} to {:?}", enemy_pos.0, closest_player);

                for step in path.iter() {
                    println!("{:?}", step);
                }

                if let Some(next_step) = path.get(1) {
                    map.remove_entity(enemy_pos.0);
                    map.add_entity_ivec3(*next_step, Tile::Enemy(enemy_entity));

                    enemy_pos.0 = *next_step;
                    transform.translation = Vec3::new(next_step.x as f32, next_step.y as f32, next_step.z as f32);
                }
            }
        }
    }
}

use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;

#[derive(Eq, PartialEq)]
struct Node {
    pos: IVec3,
    f_score: i32,
}

// Implementing Ord for BinaryHeap (it is a max-heap by default)
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score) // Reverse order to make it a min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn astar(start: IVec3, goal: IVec3, map: &Map) -> Vec<IVec3> {
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();
    let mut g_score = HashMap::new();
    let mut f_score = HashMap::new();

    open_set.push(Node {
        pos: start,
        f_score: heuristic(start, goal),
    });
    g_score.insert(start, 0);
    f_score.insert(start, heuristic(start, goal));

    while let Some(current_node) = open_set.pop() {
        let current = current_node.pos;

        if current == goal {
            return reconstruct_path(came_from, current);
        }

        closed_set.insert(current);

        for neighbor in get_valid_neighbors(current, map) {
            if closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g_score = g_score[&current] + 1;

            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g_score);
                f_score.insert(neighbor, tentative_g_score + heuristic(neighbor, goal));

                if !open_set.iter().any(|node| node.pos == neighbor) {
                    open_set.push(Node {
                        pos: neighbor,
                        f_score: f_score[&neighbor],
                    });
                }
            }
        }
    }

    Vec::new() // Return an empty path if no valid path is found
}


// Heuristic function for A*
fn heuristic(a: IVec3, b: IVec3) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
}

// Reconstruct the path
fn reconstruct_path(came_from: HashMap<IVec3, IVec3>, mut current: IVec3) -> Vec<IVec3> {
    let mut total_path = vec![];
    while let Some(&prev) = came_from.get(&current) {
        current = prev;
        total_path.push(current);
    }
    total_path.reverse();
    total_path
}

fn get_valid_neighbors(position: IVec3, map:&Map) -> Vec<IVec3> {
    let mut neighbors = Vec::new();

    let directions = [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ];

    for dir in directions.iter() {
        let neighbor_pos = position + *dir;
        if map.can_move(neighbor_pos) {
            neighbors.push(neighbor_pos);
        }
    }

    neighbors
}
