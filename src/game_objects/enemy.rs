use bevy::prelude::*;
use crate::game_objects::humanoid::{Health, Position};
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    position: Position,
    health: Health,
    replicated: Replicated,
}

impl EnemyBundle {
    pub fn new(health: u128, pos_vec: IVec3) -> Self {
        Self {
            position: Position(pos_vec),
            enemy: Enemy,
            health: Health::new(health),
            replicated: Replicated,
        }
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct Enemy;
