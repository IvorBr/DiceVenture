use bevy::prelude::*;
use std::collections::HashMap;

pub mod game_objects;

pub mod preludes;
use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;

pub mod plugins;
use plugins::network_plugin::NetworkPlugin;

mod constants;

use std::collections::BinaryHeap;
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

#[derive(Component)]
struct CameraMarker;

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
                .after(ClientSet::Receive),
            update_position,
        )
    )
    .add_systems(Update, (read_input, apply_movement, move_enemies, update_camera))
    .run();
}

fn update_camera(
    mut camera_query: Query<&mut Transform, With<CameraMarker>>, 
    players: Query<(&Player, &Transform), (With<Player>, Without<CameraMarker>)>,          
    client: Res<RepliconClient>,                                  
) {
    let client_id = client.id();

    for (player, player_transform) in players.iter() {
        if (client_id.is_some() && player.0 == client_id.unwrap()) || (!client_id.is_some() && player.0 == ClientId::SERVER) {
            if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                camera_transform.translation = Vec3::new(
                    player_transform.translation.x,
                    player_transform.translation.y + 10.0,  
                    player_transform.translation.z + 10.0,
                );

                camera_transform.look_at(
                    player_transform.translation, 
                    Vec3::Y
                );
            }
        }
    }
}

    fn update_map(mut map_events: EventReader<MapUpdate>,
        mut map: ResMut<Map>,
        mut meshes: ResMut<Assets<Mesh>>, 
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut commands: Commands
    ) {
        for event in map_events.read() {
            match event.0 {
                UpdateType::LoadTerrain => {
                    let terrain_id = commands.spawn(PbrBundle {
                        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                        material: materials.add(Color::srgb_u8(100, 255, 100)),
                        transform: Transform::from_xyz(event.1.x as f32, 0.0, event.1.z as f32),
                        ..Default::default()
                    }).id();
                    
                    //event.3 has the tile type, currently hard coded...
                    map.add_entity_ivec3(event.1, Tile::new(TileType::Terrain, terrain_id));
                }
                UpdateType::UnloadTerrain => {
                    if let Some(chunk) = map.chunks.get(&event.1) {
                        let mut entities_to_despawn = Vec::new();
        
                        for tile in &chunk.tiles {
                            if commands.get_entity(tile.entity).is_some() {
                                entities_to_despawn.push(tile.entity);
                            }
                        }
                    
                        for entity in entities_to_despawn {
                            commands.entity(entity).despawn();
                        }
                        
                        map.chunks.remove(&event.1);
                    }
                }
            }   
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
            MoveTimer(Timer::from_seconds(0.7, TimerMode::Repeating)))
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
        CameraMarker,
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
                
                if let Some(next_step) = path.get(1) {
                    map.remove_entity(enemy_pos.0);
                    map.add_entity_ivec3(*next_step, Tile::new(TileType::Enemy, enemy_entity));

                    enemy_pos.0 = *next_step;
                    transform.translation = Vec3::new(next_step.x as f32, next_step.y as f32, next_step.z as f32);
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

