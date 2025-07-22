use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt, Channel, ClientTriggerAppExt, ClientTriggerExt, FromClient, SendMode, ServerTriggerAppExt, ServerTriggerExt, ToClients};
use crate::components::character::LocalPlayer;
use crate::OverworldSet;
use crate::components::overworld::*;
use crate::plugins::camera::NewCameraTarget;

pub struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app
        .replicate::<Ship>()
        .add_client_trigger::<ClientShipPosition>(Channel::Unreliable)
        .add_server_trigger::<ServerShipPosition>(Channel::Unreliable)
        .add_observer(server_ship_move_update)
        .add_observer(client_ship_move_update)
        .add_systems(
            Update,
            (   (
                user_ship_movement,
                spawn_overworld_ship,
                ).in_set(OverworldSet),
            )
        );
    }
}

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
                Transform::from_xyz(0.0, 0.3, 0.75),
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
    mut ship: Query<(Entity, &mut Transform), (With<Ship>, With<LocalPlayer>)>,
    mut commands: Commands
) {
    if let Ok((entity, mut ship_transform)) = ship.get_single_mut() {
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
            
            commands.client_trigger_targets(
                ClientShipPosition(ship_transform.translation),
                entity
            );
        }
    }
}

fn client_ship_move_update(
    trigger: Trigger<ServerShipPosition>,
    mut ships: Query<&mut Transform, With<Ship>>,
){
    if let Ok(mut transform) = ships.get_mut(trigger.target()) {
        transform.translation = trigger.position;
    }
}

fn server_ship_move_update(
    trigger: Trigger<FromClient<ClientShipPosition>>,
    mut commands: Commands
) {
    commands.server_trigger_targets(ToClients {
            mode: SendMode::BroadcastExcept(trigger.client_entity),
            event: ServerShipPosition {
                client_entity: trigger.client_entity,
                position: trigger.0,
            }
        },
        trigger.target()
    ); 
}