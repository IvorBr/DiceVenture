use bevy::prelude::*;
use crate::objects::humanoid::{Health, Position};
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;
use std::cmp::Ordering;

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

#[derive(Component)]
pub struct MoveTimer(pub Timer);

#[derive(Eq, PartialEq)]
pub struct Node {
    pub pos: IVec3,
    pub f_score: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
