use bevy::prelude::*;
use bevy::winit::{UpdateMode::Continuous, WinitSettings};
use serde::Deserialize;
use serde::Serialize;
use crate::components::enemy::SnakePart;
use crate::components::humanoid::AttackAnimation;
use crate::components::humanoid::AttackDirection;
use crate::components::island::EnteredIsland;
use crate::components::island::LeaveIsland;
use crate::components::island::OnIsland;
use crate::components::island_maps::IslandMaps;
use crate::components::overworld::ClientShipPosition;
use crate::components::overworld::ServerShipPosition;
use crate::components::overworld::Ship;
use crate::components::player::LocalPlayer;
use crate::preludes::network_preludes::*;
use crate::preludes::humanoid_preludes::*;
use crate::GameState;
use crate::CHUNK_SIZE;

use clap::Parser;

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
pub struct OwnedBy(pub Entity);

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(WinitSettings {
            focused_mode: Continuous,
            unfocused_mode: Continuous,
        })
        .add_plugins((RepliconPlugins, RepliconRenetPlugins))
        .init_resource::<Cli>()
        .insert_resource(IslandMaps::new())
        .add_server_trigger::<MakeLocal>(Channel::Ordered)
        .add_observer(client_connected)
        .add_observer(client_disconnected)
        .add_observer(make_local)
        .add_server_event::<MapUpdate>(Channel::Ordered)
        .replicate::<OwnedBy>()
        .add_systems(Startup,
            read_cli.map(Result::unwrap)
        );
    }
}

// fn load_chunks(
//     map: Res<IslandMaps>,
//     mut map_update_events: EventWriter<ToClients<MapUpdate>>,
//     players: Query<&Position, With<Player>>
// ) {
//     let mut chunks_unload : Vec<IVec3> = vec![];
    
//     for chunk_pos in map.chunks.keys() {
//         let mut unload : bool = true;
        
//         for player_pos in players.iter() {
//             let player_chunk_pos = map.world_to_chunk_coords(player_pos.0);

//             if (player_chunk_pos - *chunk_pos).length_squared() <= 2 {
//                 unload = false;
//                 break;
//             }
//         }

//         if unload {
//             chunks_unload.push(*chunk_pos);
//         }
//     }

//     for chunk in chunks_unload {
//         map_update_events.send(ToClients {
//             mode: SendMode::Broadcast,
//             event: MapUpdate(UpdateType::UnloadTerrain, chunk, 0),
//         });
//     }

//     for position in players.iter() {
//         let chunk_pos = map.world_to_chunk_coords(position.0);
//         let load_radius = 1;

//         for chunk_x in -load_radius..=load_radius {
//             for chunk_z in -load_radius..=load_radius {
//                 let neighbor_chunk_pos = chunk_pos + IVec3::new(chunk_x, 0, chunk_z);
//                 if !map.chunks.get(&neighbor_chunk_pos).is_some() {
//                     let base_x = neighbor_chunk_pos.x * CHUNK_SIZE;
//                     let base_z = neighbor_chunk_pos.z * CHUNK_SIZE;
                    
//                     for x in 0..16 {
//                         for z in 0..16 {
//                             let pos_x = x + base_x;
//                             let pos_z = z + base_z;
                            
//                             map_update_events.send(ToClients {
//                                 mode: SendMode::Broadcast,
//                                 event: MapUpdate(UpdateType::LoadTerrain, IVec3::new(pos_x,0,pos_z), 0),
//                             });
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

#[derive(Event, Serialize, Deserialize)]
pub struct MakeLocal;

fn read_cli(
    mut commands: Commands,
    cli: Res<Cli>,
    channels: Res<RepliconChannels>,
    mut state: ResMut<NextState<GameState>>
) -> Result<(), Box<dyn Error>> {
    const PROTOCOL_ID: u64 = 0;

    match *cli {
        Cli::SinglePlayer => {
            commands.spawn((
                Ship,
                OwnedBy(SERVER),
                LocalPlayer
            ));

            state.set(GameState::Overworld);
        }
        Cli::Server { port } => {
            let server_channels_config = channels.server_configs();
            let client_channels_config = channels.client_configs();

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

            commands.spawn((
                Ship,
                OwnedBy(SERVER),
                LocalPlayer
            ));

            state.set(GameState::Overworld);
        }
        Cli::Client { port, ip } => {
            let server_channels_config = channels.server_configs();
            let client_channels_config = channels.client_configs();

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
                Text(format!("Client: {client_id}")),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor::WHITE,
            ));

            state.set(GameState::Overworld);
        }
    }

    Ok(())
}

fn make_local(
    trigger: Trigger<MakeLocal>, 
    mut commands: Commands,
) {
    commands.entity(trigger.entity()).insert(LocalPlayer);
}

fn client_connected(
    trigger: Trigger<OnAdd, ConnectedClient>, 
    mut commands: Commands
) {
    info!("{:?} connected", trigger.entity());

    let boat_entity = commands.spawn((
        Ship,
        OwnedBy(trigger.entity())
    )).id();

    commands.server_trigger_targets(
        ToClients {
            mode: SendMode::Direct(trigger.entity()),
            event: MakeLocal,
        },
        boat_entity,
    );
}

fn client_disconnected(
    trigger: Trigger<OnRemove, ConnectedClient>,
) {
    info!("{:?} disconnected", trigger.entity());
}

const PORT: u16 = 5000;

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