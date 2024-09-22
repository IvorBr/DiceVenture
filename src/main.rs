use bevy::prelude::*;

pub mod game_objects;

pub mod preludes;
use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;

pub mod plugins;
use plugins::network_plugin::NetworkPlugin;

#[derive(Component)]
struct MyCameraMarker;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, NetworkPlugin))
        .add_systems(Startup, (server_setup.run_if(resource_exists::<RenetServer>), setup))
        .add_systems(PreUpdate,
            (
                death_check.run_if(server_running).before(remove_entities),
                (init_player, init_enemy, remove_entities).after(ClientSet::Receive),
                update_map
                    .after(ServerSet::Receive)
                    .run_if(client_connected),
                update_position)
            )
        .add_systems(Update, (read_input, apply_movement))
        .run();
}

fn server_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<Map>,
    mut map_update_events: EventWriter<ToClients<MapUpdate>>){
    for x in -5..5 {
        for z in -5..5 {
            let ref_id = commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::srgb_u8(100, 255, 100)),
                transform: Transform::from_xyz(x as f32, 0.0, z as f32),
                ..Default::default()
            }).id().index();
            
            map.add_entity(x, 0, z, Tile::Terrain);

            map_update_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: MapUpdate(IVec3::new(x,0,z), ref_id, Tile::Terrain),
            });
        }
    }
    let enemy_id = commands.spawn(EnemyBundle::new(5, IVec3::new(4, 1, 4))).id();
    map.add_entity(4, 1, 4, Tile::Enemy(enemy_id));
}

fn update_map(mut map_events: EventReader<MapUpdate>,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands) {
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
        commands.entity(entity).insert(
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::srgb_u8(255, 100, 100)),
                transform: Transform::from_xyz(position.0.x as f32, position.0.y as f32, position.0.z as f32),
                ..Default::default()
        });
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

fn setup(mut commands: Commands) {
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
            input: Res<ButtonInput<KeyCode>>) {
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