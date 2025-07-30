use bevy::prelude::*;

use crate::components::humanoid::ActionState;
use crate::components::humanoid::Status;
use crate::components::humanoid::StatusFlags;
use crate::components::humanoid::Stunned;
use crate::components::island::OnIsland;
use crate::components::character::LocalPlayer;
use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;
use crate::components::island_maps::IslandMaps;
use crate::GameState;
use crate::IslandSet;

pub struct HumanoidPlugin;
impl Plugin for HumanoidPlugin {
    fn build(&self, app: &mut App) {
        app
        .replicate::<RemoveEntity>()
        .add_systems(PreUpdate,
        (
            standard_death_check.run_if(server_running),
            animate_movement
        ).in_set(IslandSet))
        .add_systems(PreUpdate, (sync_status_flags_system, status_flags_to_actionstate_system).chain())
        .add_systems(Update, (remove_entities).after(ClientSet::Receive));
    }
}

fn standard_death_check(
    mut commands: Commands,
    entities: Query<(&Health, Entity), Without<Enemy>>,
) {
    for (health, entity) in &entities {
        if health.get() == 0 {
            commands.entity(entity).insert(RemoveEntity);
        }
    }
}

fn remove_entities(
    mut commands: Commands,
    entities: Query<(Entity, &Position, &OnIsland, Option<&LocalPlayer>), With<RemoveEntity>>,
    mut islands: ResMut<IslandMaps>,
    mut state: ResMut<NextState<GameState>>
) {
    for (entity, position, island, local_player) in entities.iter() {
        islands.get_map_mut(island.0).map(|map| map.remove_entity(position.0));
        println!("Despawning entity: {:?}", entity);
        commands.entity(entity).despawn();
        
        if local_player.is_some() {
            state.set(GameState::Overworld);
        }
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