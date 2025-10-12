use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientTriggerAppExt, FromClient, SendMode, ServerTriggerAppExt, ServerTriggerExt, ToClients};

use crate::{components::{character::LocalPlayer, enemy::Enemy, overworld::{ClientShipPosition, Island, OverworldRoot, ServerShipPosition}, ship::{PatrolIsland, Ship}}, plugins::camera::NewCameraTarget, OverworldSet};

pub struct OverworldShipPlugin;
impl Plugin for OverworldShipPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_client_trigger::<ClientShipPosition>(Channel::Unreliable)
        .add_server_trigger::<ServerShipPosition>(Channel::Unreliable)
        .add_observer(server_ship_move_update)
        .add_observer(client_ship_move_update)
        .add_systems(
            Update,
            (
                spawn_ships,
                (
                spawn_overworld_ship,
                move_enemy_patrol_ship,
                ).in_set(OverworldSet),
            )
        );
    }
}

fn spawn_ships(
    mut commands: Commands,
    island_query: Query<Entity, Added<Island>>
){
    for island in island_query.iter() {
        commands.spawn((
            Ship,
            Enemy,
            PatrolIsland(island)
        ));
    }
}

fn spawn_overworld_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ships: Query<(Entity, Option<&LocalPlayer>), (With<Ship>, Without<Transform>)>,
    world_root_query: Query<Entity, With<OverworldRoot>>,
) {
    if let Ok(overworld_root) = world_root_query.single() {
        for (entity, local) in ships.iter_mut() {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.65, 0.45, 0.25),
                    ..Default::default()
                })),
                Transform::from_xyz(0.0, 0.3, 0.75),
                Visibility::Inherited
            )).insert(ChildOf(overworld_root));
    
            if local.is_some() { 
                commands.entity(entity).insert(NewCameraTarget);
            }
        }
    }
}

//TODO: Have ships move between allegiance islands, or patrol around them
fn move_enemy_patrol_ship(
    mut commands: Commands,
    ships: Query<(Entity, &PatrolIsland), (With<Ship>, With<Enemy>)>,
    island_transform_query: Query<&Transform, With<Island>>
) {
    for (ship_entity, patrol_island) in ships.iter() {
        if let Ok(island_transform) = island_transform_query.get(patrol_island.0) {
            let new_position = island_transform.translation - 20.0 * Vec3::X;

            commands.server_trigger_targets(
                ToClients {
                    mode: SendMode::Broadcast,
                    event: ServerShipPosition {
                        position: new_position,
                    }
                },
                ship_entity
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
                position: trigger.0,
            }
        },
        trigger.target()
    ); 
}

fn enter_ship(

){
    
}