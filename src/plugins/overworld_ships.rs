use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientEventAppExt, FromClient, SendMode, ServerEventAppExt, ServerTriggerExt, ToClients};

use crate::{
    components::{
        character::LocalPlayer, core::Initialized, island::EnterPosition, 
        overworld::{ClientShipPosition, Island, OverworldPosition, OverworldRoot, ServerShipPosition}, 
        ship::{PatrolIsland, Ship}
    }, 
    plugins::camera::NewCameraTarget
};

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
                update_ship_position,
                spawn_overworld_ship,
                spawn_ships,
                move_enemy_patrol_ship
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
    ships: Query<(Entity, &PatrolIsland), Added<Ship>>,
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

fn update_ship_position(
    time: Res<Time>,
    mut ships: Query<(&mut Transform, &OverworldPosition), With<Ship>>,
) {  
    let dt = time.delta_secs();

    for (mut tf, o_pos) in ships.iter_mut() {
        let to_target = o_pos.target - tf.translation;
        let dist = to_target.length();
        if dist < 0.001 { continue; }

        // Exponential catch-up factor: 1 - e^(-k * dt)
        let catchup = 1.0 - (-10.0 * dt).exp();
        let step = to_target * catchup;

        tf.translation += step;
    }
}

fn client_ship_move_update(
    trigger: On<ServerShipPosition>,
    mut ships: Query<&mut OverworldPosition, With<Ship>>,
) {
    if let Ok(mut o_pos) = ships.get_mut(trigger.entity) {
        o_pos.target = trigger.position;
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