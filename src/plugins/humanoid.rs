use bevy::prelude::*;

use crate::components::humanoid::ActionState;
use crate::components::enemy::SnakePart;
use crate::components::island::OnIsland;
use crate::components::player::LocalPlayer;
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
                death_check.run_if(server_running).before(remove_entities),
                (remove_entities).after(ClientSet::Receive),
                animate_movement
            ).in_set(IslandSet)
        );
    }
}

fn death_check(
    mut commands: Commands,
    entities: Query<(&Health, Entity), Or<(With<Player>, With<Enemy>)>>,
    snake_parts: Query<&SnakePart>
) {
    for (health, entity) in &entities {
        if health.get() == 0 {
            println!("{}, {}", entity, health.get());
            commands.entity(entity).insert(RemoveEntity);
            
            if let Ok(mut current) = snake_parts.get(entity) {
                while let Some(next_entity) = current.next {
                    commands.entity(next_entity).insert(RemoveEntity);
                    current = match snake_parts.get(next_entity) {
                        Ok(snake) => snake,
                        _ => break,
                    };
                }
            }
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
        islands.get_map_mut(island.0).remove_entity(position.0);
        println!("Despawning entity: {:?}", entity);
        commands.entity(entity).despawn_recursive();
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
