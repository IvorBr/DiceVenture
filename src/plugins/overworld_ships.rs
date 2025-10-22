use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientEventAppExt, FromClient, SendMode, ServerEventAppExt, ServerTriggerExt, ToClients};

use crate::{components::{character::LocalPlayer, core::{Initialized, Seed}, enemy::Enemy, island::{EnterPosition, EnteredIsland, GenerateIsland, VisualizeIsland}, overworld::{ClientShipPosition, Entered, Island, LocalIsland, OverworldRoot, ServerShipPosition}, ship::{PatrolIsland, Ship}}, plugins::camera::NewCameraTarget, GameState, OverworldSet};

pub struct OverworldShipPlugin;
impl Plugin for OverworldShipPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_mapped_client_event::<ClientShipPosition>(Channel::Unreliable)
        .add_mapped_server_event::<ServerShipPosition>(Channel::Unreliable)
        .add_observer(server_ship_move_update)
        .add_observer(client_ship_move_update)
        .add_systems(
            Update,
            (
                spawn_ships,
                (spawn_overworld_ship, move_enemy_patrol_ship).in_set(OverworldSet),
            )
        );
    }
}

fn spawn_ships(
    mut commands: Commands,
    island_query: Query<Entity, Added<Island>>,
){
    for island in island_query.iter() {
        commands.spawn((
            Ship,
           // Enemy,
            PatrolIsland(island),
            EnterPosition(Vec3::ZERO)
        ));
    }
}

fn spawn_overworld_ship(
    mut commands: Commands,
    mut ships: Query<(Entity, Option<&LocalPlayer>), (With<Ship>, Without<Transform>)>,
    world_root_query: Query<Entity, With<OverworldRoot>>,
) {
    if let Ok(overworld_root) = world_root_query.single() {
        for (entity, local) in ships.iter_mut() {
            commands.entity(entity).insert((
                Initialized,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Visibility::Inherited
            )).insert(ChildOf(overworld_root));
    
            if local.is_some() { 
                commands.entity(entity).insert(NewCameraTarget);
            }
        }
    }
}

//TODO: Have ships move between allegianced islands, or patrol around them
fn move_enemy_patrol_ship(
    mut commands: Commands,
    ships: Query<(Entity, &PatrolIsland), (With<Ship>)>,
    island_transform_query: Query<&Transform, With<Island>>
) {
    for (ship_entity, patrol_island) in ships.iter() {
        if let Ok(island_transform) = island_transform_query.get(patrol_island.0) {
            let new_position = island_transform.translation - 20.0 * Vec3::X;

            commands.server_trigger(
                ToClients {
                    mode: SendMode::Broadcast,
                    message: ServerShipPosition {
                        position: new_position,
                        entity: ship_entity
                    }
                },
            ); 
        }
    }
}

fn client_ship_move_update(
    trigger: On<ServerShipPosition>,
    mut ships: Query<&mut Transform, With<Ship>>,
){
    if let Ok(mut transform) = ships.get_mut(trigger.entity) {
        transform.translation = trigger.position;
    }
}

fn server_ship_move_update(
    trigger: On<FromClient<ClientShipPosition>>,
    mut commands: Commands
) {
    commands.server_trigger(ToClients {
            mode: SendMode::BroadcastExcept(trigger.client_id),
            message: ServerShipPosition {
                position: trigger.position,
                entity: trigger.entity
            }
        },
    ); 
}