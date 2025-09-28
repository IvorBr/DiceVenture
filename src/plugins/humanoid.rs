use bevy::prelude::*;

use crate::components::humanoid::ActionState;
use crate::components::humanoid::PositionUpdate;
use crate::components::humanoid::ServerPositionUpdate;
use crate::components::humanoid::Status;
use crate::components::humanoid::StatusFlags;
use crate::components::humanoid::Stunned;
use crate::components::humanoid::ViewDirection;
use crate::components::island::LeaveIsland;
use crate::components::island::OnIsland;
use crate::plugins::network::OwnedBy;
use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;
use crate::components::island_maps::IslandMaps;
use crate::IslandSet;

pub struct HumanoidPlugin;
impl Plugin for HumanoidPlugin {
    fn build(&self, app: &mut App) {
        app
        .replicate::<RemoveEntity>()
        .replicate::<Health>()
        .add_event::<PositionUpdate>()
        .add_server_trigger::<ServerPositionUpdate>(Channel::Ordered)
        .add_observer(position_trigger)
        .add_systems(PreUpdate,
        (
            ((player_death_check, position_change_event).run_if(server_running),
            (animate_movement,animate_view_direction).in_set(IslandSet)),
            (sync_status_flags_system, status_flags_to_actionstate_system).chain(),
        ))
        .add_systems(Update, (remove_entities).run_if(server_running));
    }
}

fn player_death_check(
    mut commands: Commands,
    entities: Query<(&Health, Entity, &OwnedBy, &OnIsland), Without<Enemy>>,
    mut leave_island_event: EventWriter<ToClients<LeaveIsland>>,
) {
    for (health, entity, owner, island) in entities.iter() {
        if health.get() == 0 {
            commands.entity(entity).insert(RemoveEntity);
            leave_island_event.write(ToClients { mode: SendMode::Direct(owner.0), event: LeaveIsland(island.0) });
        }
    }
}

// TODO: Currently only runs on the server, as the deletion is already being replicated. This should be improved to where the client decides what to despawn
fn remove_entities(
    mut commands: Commands,
    entities: Query<(Entity, &Position, &OnIsland), With<RemoveEntity>>,
    mut islands: ResMut<IslandMaps>
) {
    for (entity, position, island) in entities.iter() {
        islands.get_map_mut(island.0).map(|map| {map.remove_entity(position.0); map.entities.remove(&entity)});
        println!("Despawning entity: {:?}", entity);
        commands.entity(entity).despawn();
    }
}

fn position_change_event(
    mut commands: Commands,
    mut event: EventReader<PositionUpdate>,
    mut entity_query: Query<(&mut Position, &OnIsland, Option<&Character>)>,
    mut island_maps: ResMut<IslandMaps>
) {
    for PositionUpdate { new_position, entity } in event.read() {
        if let Ok((mut entity_position, island, character)) = entity_query.get_mut(*entity) {
            if let Some(map) = island_maps.get_map_mut(island.0) {
                
                let mut tile_type = TileType::Enemy;
                if character.is_some() {
                    tile_type = TileType::Player;
                }

                map.remove_entity(entity_position.0);
                map.add_entity_ivec3(*new_position, Tile::new(tile_type, *entity));
                entity_position.0 = *new_position;

                commands.server_trigger_targets(
                    ToClients {
                        mode: SendMode::BroadcastExcept(SERVER),
                        event: ServerPositionUpdate { position: *new_position } ,
                    },
                    *entity,
                );
            }
        }
    }
}

fn position_trigger(
    trigger: Trigger<ServerPositionUpdate>,
    mut entity_query: Query<(&mut Position, &mut ViewDirection, &OnIsland, Option<&Character>)>,
    mut island_maps: ResMut<IslandMaps>
) {
    if let Ok((mut position, mut view_direction, island, character)) = entity_query.get_mut(trigger.target()) {
        if let Some(map) = island_maps.get_map_mut(island.0) {
            
            let mut tile_type = TileType::Enemy;
            if character.is_some() {
                tile_type = TileType::Player;
            }

            map.remove_entity(position.0);
            map.add_entity_ivec3(trigger.position, Tile::new(tile_type, trigger.target()));
            
            view_direction.0 = ((position.0 - trigger.position).as_vec3() * Vec3::new(1.0, 0.0, 1.0)).normalize_or_zero().round().as_ivec3();

            position.0 = trigger.position;
        }
    }
}

fn animate_view_direction(
    mut q: Query<(&ViewDirection, &mut Transform), Changed<ViewDirection>>,
) {
    for (view_dir, mut transform) in &mut q {
        let target = view_dir.0.as_vec3();
        transform.rotation = Quat::from_rotation_arc(-Vec3::Z, target);
    }
}

fn animate_movement(
    mut moved_entities: Query<(&Position, &mut Transform, &mut ActionState)>, 
    time: Res<Time>
) {
    for (position, mut transform, mut action_state) in &mut moved_entities {
        let target = position.0.as_vec3();
        let current = transform.translation;

        if current.distance(target) > 0.01 {
            transform.translation = current.lerp(target, time.delta_secs() * 10.0);
            *action_state = ActionState::Moving;

        } else {
            transform.translation = target;
            *action_state = ActionState::Idle;
        }
    }
}

pub fn sync_status_flags_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut StatusFlags,
        Option<&mut Stunned>,
    )>,
) {
    for (entity, mut flags, stunned_opt) in &mut query {
        let mut status = Status::empty();

        if let Some(mut stunned) = stunned_opt {
            if stunned.timer.tick(time.delta()).finished() {
                commands.entity(entity).remove::<Stunned>();
            } else {
                status |= Status::STUNNED;
            }
        }

        flags.0 = status;
    }
}

pub fn status_flags_to_actionstate_system(
    mut query: Query<(&StatusFlags, &mut ActionState)>,
) {
    for (flags, mut action_state) in &mut query {
        if flags.0.contains(Status::STUNNED) {
            *action_state = ActionState::Stunned;
        } else if *action_state == ActionState::Stunned {
            *action_state = ActionState::Idle;
        }
    }
}