use bevy::prelude::*;

use crate::components::{character::Character, enemy::{EnemyState, PassiveAggro, RangeAggro}, humanoid::Position};
pub struct AggressionPlugin;

impl Plugin for AggressionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, passive_aggro_system)
            .add_systems(Update, range_aggro_system);
    }
}

fn find_closest_in_range(players: &Query<(&Position, Entity), With<Character>>, enemy_pos: &Position, range: i32) -> Option<Entity> {
    let mut closest_player: Option<Entity> = None;
    let mut closest_distance: i32 = i32::MAX;

    for (player_pos, player_entity) in players.iter() {
        let distance = player_pos.get().distance_squared(enemy_pos.get());

        if distance <= range * range && distance < closest_distance {
            closest_player = Some(player_entity);
            closest_distance = distance;
        }
    }
    closest_player
}

fn passive_aggro_system(mut _enemies: Query<&mut EnemyState, With<PassiveAggro>>) {
    // Attack if you are attacked, still need to be created
}

fn range_aggro_system(
    mut enemies: Query<(&Position, &RangeAggro, &mut EnemyState)>,
    players: Query<(&Position, Entity), With<Character>>,
    mut commands: Commands
) {
    for (enemy_pos, aggro, mut state) in enemies.iter_mut() {

        if !matches!(*state, EnemyState::Attacking(_)) {
            if let Some(player) = find_closest_in_range(&players, enemy_pos, aggro.0) {
                *state = EnemyState::Attacking(player);
            } else {
                *state = EnemyState::Idle;
            }
        }
        else {
            if let EnemyState::Attacking(target) = *state {
                if commands.get_entity(target).is_ok() {
                    *state = EnemyState::Idle;
                }
            }
        }
    }
}
