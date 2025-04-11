use bevy::prelude::*;
use bevy_replicon::prelude::{FromClient, SendMode, ToClients};
use crate::components::player::LocalPlayer;
use crate::OverworldSet;
use crate::components::overworld::*;
use crate::plugins::camera::NewCameraTarget;

use super::network::OwnedBy;

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
    mut ships: Query<(Entity, Option<&LocalPlayer>), (With<Ship>, Without<Transform>)>,
    world_root_query: Query<Entity, With<OverworldRoot>>,
) {
    if let Ok(overworld_root) = world_root_query.get_single() {
        for (entity, local) in ships.iter_mut() {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.65, 0.45, 0.25),
                    ..Default::default()
                })),
                Transform::from_xyz(0.0, 0.0, 0.75),
                Visibility::Inherited
            )).set_parent(overworld_root);
    
            if local.is_some() { 
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
    mut ships: Query<(&mut Transform, &OwnedBy), With<Ship>>,
){
    for ServerShipPosition { client_entity, position } in ship_move_reader.read() {
        for (mut transform, owner) in ships.iter_mut() {
            if owner.0 == *client_entity {
                println!("MOVING SHIP: {:?}", *client_entity);
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
    for FromClient { client_entity, event } in ship_move_reader.read() {
        ship_move_writer.send(ToClients {
            mode: SendMode::BroadcastExcept(*client_entity),
            event: ServerShipPosition {
                client_entity: *client_entity,
                position: event.0,
            }
        });
    }
}