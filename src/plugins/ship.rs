use bevy::prelude::*;
use bevy_replicon::prelude::{FromClient, RepliconClient, SendMode, ToClients};
use crate::components::player::LocalPlayer;
use crate::{GameState, OverworldSet};
use crate::components::overworld::*;
use crate::plugins::camera::NewCameraTarget;
use bevy_replicon::core::ClientId;

pub struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(
            Update,
            (   (
                user_ship_movement,
                spawn_overworld_ship,
                client_ship_move_update,
                server_ship_move_update
                ).in_set(OverworldSet),
            )
        );
    }
}

//TODO: function spawn ships without transforms, so they show up locally.
fn spawn_overworld_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ships: Query<(Entity, &Ship), (With<Ship>, Without<Transform>)>,
    client: Res<RepliconClient>,
    world_root_query: Query<Entity, With<OverworldRoot>>,
) {
    if let Ok(overworld_root) = world_root_query.get_single() {
        let client_id = client.id();
        for (entity, ship) in ships.iter_mut() {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.65, 0.45, 0.25),
                    ..Default::default()
                })),
                Transform::from_xyz(0.0, 0.0, 0.75),
                Visibility::Inherited
            )).set_parent(overworld_root);
    
            if client_id.map_or(ship.0 == ClientId::SERVER, |id| id == ship.0) {
                commands.entity(entity).insert(LocalPlayer);
                commands.entity(entity).insert(NewCameraTarget);
            }
        }
    }
}

fn user_ship_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ship: Query<&mut Transform, (With<Ship>, With<LocalPlayer>, Without<Ocean>)>,
    mut ocean: Query<&mut Transform, (Without<Ship>, With<Ocean>)>,
    mut ship_move_writer: EventWriter<ClientShipPosition>,
) {
    if let Ok(mut ship_transform) = ship.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec3::ZERO {
            let speed = 5.0;
            ship_transform.translation += direction.normalize() * speed * time.delta_secs();
            
            for mut ocean_transform in &mut ocean {
                ocean_transform.translation.x = ship_transform.translation.x;
                ocean_transform.translation.z = ship_transform.translation.z;
            }

            ship_move_writer.send(ClientShipPosition(ship_transform.translation));
        }
    }
}

fn client_ship_move_update(
    mut ship_move_reader: EventReader<ServerShipPosition>,
    mut ships: Query<(&mut Transform, &Ship)>,
){
    for ServerShipPosition { client_id, position } in ship_move_reader.read() {
        for (mut transform, ship) in ships.iter_mut() {
            if ship.0 == *client_id {
                transform.translation = *position;
                break;
            }
        }
    }
}

fn server_ship_move_update(
    mut ship_move_reader: EventReader<FromClient<ClientShipPosition>>,
    mut ship_move_writer: EventWriter<ToClients<ServerShipPosition>>

) {
    for FromClient { client_id, event } in ship_move_reader.read() {
        ship_move_writer.send(ToClients {
            mode: SendMode::BroadcastExcept(*client_id),
            event: ServerShipPosition {
                client_id: *client_id,
                position: event.0,
            }
        });
    }
}