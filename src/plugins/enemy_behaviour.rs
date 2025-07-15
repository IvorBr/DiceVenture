use bevy::prelude::*;

use crate::components::{enemy::{EnemyState, PassiveAggro, RangeAggro}, humanoid::Position, player::Player};
pub struct AggressionPlugin;

impl Plugin for AggressionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, passive_aggro_system)
            .add_systems(Update, range_aggro_system);
    }
}

fn find_closest_in_range(players: &Query<(&Position, Entity), With<Player>>, enemy_pos: &Position, range: i32) -> Option<Entity> {
    let mut closest_player: Option<Entity> = None;
    let mut closest_distance: i32 = i32::MAX;

    for (player_pos, player_entity) in players.iter() {
        let distance = player_pos.0.distance_squared(enemy_pos.0);

        if distance <= range * range && distance < closest_distance {
            closest_player = Some(player_entity);
            closest_distance = distance;
        }
    }
    closest_player
}

fn passive_aggro_system(mut enemies: Query<&mut EnemyState, With<PassiveAggro>>) {
    // Attack if you are attacked, still need to be created
}

fn range_aggro_system(
    mut enemies: Query<(&Position, &RangeAggro, &mut EnemyState)>,
    players: Query<(&Position, Entity), With<Player>>,
) {
    for (enemy_pos, aggro, mut state) in enemies.iter_mut() {
        if !matches!(*state, EnemyState::Attacking(_)) {
            if let Some(player) = find_closest_in_range(&players, enemy_pos, aggro.0) {
                *state = EnemyState::Attacking(player);
            } else {
                *state = EnemyState::Idle;
            }
        }
    }
}
