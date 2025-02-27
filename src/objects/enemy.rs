use bevy::prelude::*;
use crate::objects::humanoid::{Health, Position};
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;
use std::cmp::Ordering;

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub position: Position,
    pub health: Health,
    pub replicated: Replicated,
}

impl EnemyBundle {
    pub fn new_with_movement(health: u128, pos_vec: IVec3, movement: MovementType) -> Self {
        Self {
            position: Position(pos_vec),
            enemy: Enemy {
                movement,
                ..Enemy::default()
            },
            health: Health::new(health),
            replicated: Replicated,
        }
    }

    pub fn new(health: u128, pos_vec: IVec3) -> Self {
        Self::new_with_movement(health, pos_vec, MovementType::Standard)
    }

    fn default() -> Self {
        Self::new(100, IVec3::ZERO)
    }
}



#[derive(Serialize, Deserialize, Default)]
pub enum MovementType {
    #[default]
    Standard,
    Snake,
    Multi,
}

#[derive(Component, Serialize, Deserialize)]
pub struct Enemy {
    pub movement: MovementType,
    //pub attacks?
    //pub aggressionType?
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy {
            movement: MovementType::default(),
        }
    }
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

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct SnakePart {
    pub next: Option<Entity>
}
