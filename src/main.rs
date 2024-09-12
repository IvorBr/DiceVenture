use bevy::prelude::*;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

pub mod game_objects;
use game_objects::humanoid::{Health, Position, MoveDirection};
use game_objects::player::{Player, PlayerBundle};
use game_objects::enemy::{Enemy, EnemyBundle};
use game_objects::grid::{Tile, Map};

use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        ConnectionConfig, RenetClient, RenetServer,
    },
    RenetChannelsExt, RepliconRenetPlugins,
};

use clap::Parser;

#[derive(Component)]
struct MyCameraMarker;

const PORT: u16 = 5000;
const PROTOCOL_ID: u64 = 0;

#[derive(Parser, PartialEq, Resource)]
enum Cli {
    SinglePlayer,
    Server {
        #[arg(short, long, default_value_t = PORT)]
        port: u16,
    },
    Client {
        #[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
        ip: IpAddr,

        #[arg(short, long, default_value_t = PORT)]
        port: u16,
    },
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
    }
}

fn main() {
    App::new()
        .init_resource::<Cli>()
        .add_plugins((DefaultPlugins, RepliconPlugins, RepliconRenetPlugins))
        .replicate::<Player>()
        .replicate::<Position>()
        .add_client_event::<MoveDirection>(ChannelKind::Ordered)
        .insert_resource(Map::new())
        .add_systems(Startup, (read_cli.map(Result::unwrap), setup))
        .add_systems(PreUpdate, (init_player.after(ClientSet::Receive), update_position))
        .add_systems(Update, (handle_connections.run_if(server_running), read_input, apply_movement))
        .run();
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

fn read_cli(
    mut commands: Commands,
    cli: Res<Cli>,
    channels: Res<RepliconChannels>,
) -> Result<(), Box<dyn Error>> {
    match *cli {
        Cli::SinglePlayer => {
            commands.spawn(PlayerBundle::new(
                ClientId::SERVER,
                5,
                IVec3::new(0,1,0)
            ));
        }
        Cli::Server { port } => {
            let server_channels_config = channels.get_server_configs();
            let client_channels_config = channels.get_client_configs();

            let server = RenetServer::new(ConnectionConfig {
                server_channels_config,
                client_channels_config,
                ..Default::default()
            });

            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))?;
            let server_config = ServerConfig {
                current_time,
                max_clients: 10,
                protocol_id: PROTOCOL_ID,
                authentication: ServerAuthentication::Unsecure,
                public_addresses: Default::default(),
            };
            let transport = NetcodeServerTransport::new(server_config, socket)?;

            commands.insert_resource(server);
            commands.insert_resource(transport);

            commands.spawn(TextBundle::from_section(
                "Server",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            commands.spawn(PlayerBundle::new(
                ClientId::SERVER,
                5,
                IVec3::new(0,1,0)
                ));
        }
        Cli::Client { port, ip } => {
            let server_channels_config = channels.get_server_configs();
            let client_channels_config = channels.get_client_configs();

            let client = RenetClient::new(ConnectionConfig {
                server_channels_config,
                client_channels_config,
                ..Default::default()
            });

            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            let client_id = current_time.as_millis() as u64;
            let server_addr = SocketAddr::new(ip, port);
            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: PROTOCOL_ID,
                server_addr,
                user_data: None,
            };
            let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

            commands.insert_resource(client);
            commands.insert_resource(transport);

            commands.spawn(TextBundle::from_section(
                format!("Client: {client_id:?}"),
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        }
    }

    Ok(())
}

// Logs server events and spawns a new player whenever a client connects.
fn handle_connections(mut commands: Commands, 
    mut server_events: EventReader<ServerEvent>) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("{client_id:?} connected");

                commands.spawn(PlayerBundle::new(
                    *client_id,
                    5,
                    IVec3::new(-1,1,0)
                ));
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("{client_id:?} disconnected: {reason}");
            }
        }
    }
}

fn setup(mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>, 
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut map: ResMut<Map>) {
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
    
    for x in 1..4 {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(100, 255, 100)),
            transform: Transform::from_xyz(x as f32, x as f32, 0.0),
            ..Default::default()
        });
        map.add_entity(x, x, 0, Tile::Terrain);
    }
    
    for x in -5..5 {
        for z in -5..5 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::srgb_u8(100, 255, 100)),
                transform: Transform::from_xyz(x as f32, 0.0, z as f32),
                ..Default::default()
            });

            map.add_entity(x, 0, z, Tile::Terrain)
        }
    }

    let entity_id = commands.spawn((
        EnemyBundle::new(5, IVec3::new(-3, 1, -2)),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(255, 100, 100)),
            transform: Transform::from_xyz(-3.0, 1.0, -2.0),
            ..default()
        },
    )).id();

    map.add_entity(-3, 1, -2, Tile::Enemy(entity_id));
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
                map.add_entity_IVec3(new_position, Tile::Player(player_entity));

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