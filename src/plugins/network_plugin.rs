use bevy::prelude::*;
use crate::server_setup;

use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
pub struct NetworkPlugin;

use clap::Parser;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins))
        .init_resource::<Cli>()
        .insert_resource(Map::new())
        .add_client_event::<MoveDirection>(ChannelKind::Ordered)
        .add_server_event::<MapUpdate>(ChannelKind::Ordered)
        .replicate::<Player>()
        .replicate::<Position>()
        .replicate::<Enemy>()
        .replicate::<RemoveEntity>()
        .add_systems(Startup, read_cli.map(Result::unwrap).before(server_setup))
        .add_systems(Update, handle_connections.run_if(server_running));
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
    mut server_events: EventReader<ServerEvent>,
    players: Query<(Entity, &Player), With<Player>>,
    map: ResMut<Map>,
    mut map_update_events: EventWriter<ToClients<MapUpdate>>
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("{client_id:?} connected");

                commands.spawn(PlayerBundle::new(
                    *client_id,
                    5,
                    IVec3::new(-1,1,0)
                ));

                for (key, value) in map.grid.iter() {
                    if *value == Tile::Terrain {
                        map_update_events.send(ToClients {
                            mode: SendMode::Direct(*client_id),
                            event: MapUpdate(*key, 0, *value), //TODO:: NEEDS AN ACTUAL REF ID
                        });
                    }
                }
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("{client_id:?} disconnected: {reason}");
                for (entity, player) in players.iter() {
                    if player.0 == *client_id {
                        commands.entity(entity).insert(RemoveEntity);
                    }
                }
            }
        }
    }
}

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