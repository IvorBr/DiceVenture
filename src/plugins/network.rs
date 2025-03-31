use bevy::prelude::*;

use crate::components::enemy::SnakePart;
use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
use crate::CHUNK_SIZE;
use crate::IslandSet;

use clap::Parser;
pub struct NetworkPlugin;
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
        .replicate::<Shape>()
        .replicate::<SnakePart>()
        .replicate::<RemoveEntity>()
        .add_systems(Startup,
            read_cli.map(Result::unwrap)
        )
        .add_systems(Update, (
            //load_chunks.run_if(server_running), //might wanna turn on again later?
            handle_connections.run_if(server_running)
        ).in_set(IslandSet));
    }
}

fn load_chunks(
    map: Res<Map>,
    mut map_update_events: EventWriter<ToClients<MapUpdate>>,
    players: Query<&Position, With<Player>>
) {
    let mut chunks_unload : Vec<IVec3> = vec![];
    
    for chunk_pos in map.chunks.keys() {
        let mut unload : bool = true;
        
        for player_pos in players.iter() {
            let player_chunk_pos = map.world_to_chunk_coords(player_pos.0);

            if (player_chunk_pos - *chunk_pos).length_squared() <= 2 {
                unload = false;
                break;
            }
        }

        if unload {
            chunks_unload.push(*chunk_pos);
        }
    }

    for chunk in chunks_unload {
        map_update_events.send(ToClients {
            mode: SendMode::Broadcast,
            event: MapUpdate(UpdateType::UnloadTerrain, chunk, 0),
        });
    }

    for position in players.iter() {
        let chunk_pos = map.world_to_chunk_coords(position.0);
        let load_radius = 1;

        for chunk_x in -load_radius..=load_radius {
            for chunk_z in -load_radius..=load_radius {
                let neighbor_chunk_pos = chunk_pos + IVec3::new(chunk_x, 0, chunk_z);
                if !map.chunks.get(&neighbor_chunk_pos).is_some() {
                    let base_x = neighbor_chunk_pos.x * CHUNK_SIZE;
                    let base_z = neighbor_chunk_pos.z * CHUNK_SIZE;
                    
                    for x in 0..16 {
                        for z in 0..16 {
                            let pos_x = x + base_x;
                            let pos_z = z + base_z;
                            
                            map_update_events.send(ToClients {
                                mode: SendMode::Broadcast,
                                event: MapUpdate(UpdateType::LoadTerrain, IVec3::new(pos_x,0,pos_z), 0),
                            });
                        }
                    }
                }
            }
        }
    }
}

fn read_cli(
    mut commands: Commands,
    cli: Res<Cli>,
    channels: Res<RepliconChannels>,
) -> Result<(), Box<dyn Error>> {
    match *cli {
        Cli::SinglePlayer => {
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

            commands.spawn((Text::new("Server"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE)
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

            commands.spawn((
                Text::new(format!("Client: {client_id:?}")),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE)
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

                // commands.spawn(PlayerBundle::new(
                //     *client_id,
                //     5,
                //     IVec3::new(6,1,5)
                // ));

                // for (chunk_pos, chunk) in map.chunks.iter() {
                //     for i in 0..chunk.tiles.len() {
                //         let x = (i % 16) as i32;
                //         let y = ((i / 16) % 16) as i32;
                //         let z = (i / (16 * 16)) as i32;
                
                //         let world_x = chunk_pos.x * 16 + x;
                //         let world_y = chunk_pos.y * 16 + y;
                //         let world_z = chunk_pos.z * 16 + z;
                //         let tile = chunk.tiles[i];

                //         if tile.kind == TileType::Terrain {
                //             map_update_events.send(ToClients {
                //                 mode: SendMode::Direct(*client_id),
                //                 event: MapUpdate(UpdateType::LoadTerrain, IVec3::new(world_x, world_y, world_z), 0), // Include actual ref_id if needed
                //             });
                //         }
                        
                //     }
                // }
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("{client_id:?} disconnected: {reason}");

                //clean up all player stuff, curently just the player...
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