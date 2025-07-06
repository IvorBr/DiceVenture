use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;
use std::cmp::Ordering;

use super::humanoid::Humanoid;

#[derive(Component, Serialize, Deserialize)]
#[require(Replicated, Humanoid)]
pub struct Enemy;

#[derive(Component, Default, Serialize, Deserialize)]
pub enum EnemyState {
    #[default]
    Idle,
    Searching,
    Attacking(Entity),
    Fleeing,
}

#[derive(Component, Serialize, Deserialize, Default)]
pub enum Movement {
    #[default]
    Standard,
    Snake,
    Multi,
}

#[derive(Component, Serialize, Deserialize, Default)]
pub enum Aggression {
    #[default]
    Passive,
    RangeBased(i32),
}

#[derive(Component)]
pub struct MoveTimer(pub Timer);

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct Shape(pub Vec<IVec3>);

impl Shape {
    pub fn new_2x2x2()-> Self {
        Shape(vec![
            IVec3::new(0, 0, 1),
            IVec3::new(1, 0, 0),
            IVec3::new(1, 0, 1),
            
            IVec3::new(0, 1, 0),
            IVec3::new(0, 1, 1),
            IVec3::new(1, 1, 0),
            IVec3::new(1, 1, 1),
        ])
    }
}

#[derive(Eq, PartialEq)]
pub struct PathfindNode {
    pub pos: IVec3,
    pub f_score: i32,
}

impl Ord for PathfindNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for PathfindNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct SnakePart {
    pub next: Option<Entity>
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum AttackPhase {
    Windup,  
    Strike
}

#[derive(Component, Deserialize, Serialize, Clone)]
pub struct WindUp {
    pub target_pos: IVec3,
    pub timer: Timer,
    pub phase: AttackPhase,
}

#[derive(Deserialize, Event, Serialize)]
pub struct StartAttack {
    pub attack: WindUp
}