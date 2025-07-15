use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;
use std::cmp::Ordering;

use crate::plugins::attack::{AttackId, AttackSpec};

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

#[derive(Component)] 
pub struct PassiveAggro;
#[derive(Component)] 
pub struct RangeAggro(pub i32);

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

#[derive(Component, Clone, Copy)]
pub struct MoveRule {
    pub offsets : &'static [IVec3],
    pub can_climb : bool,
    pub heuristic : fn(IVec3, IVec3) -> i32,
}

#[derive(Component)]
pub struct Attacks(pub Vec<AttackId>);

pub const STANDARD: [IVec3; 4] = [ 
    IVec3::X, 
    IVec3::new(-1,0,0), 
    IVec3::Z, 
    IVec3::new(0,0,-1)
];

pub const DIAGONAL: [IVec3; 4] = [
    IVec3::new(1, 0, 1),
    IVec3::new(-1, 0, 1),
    IVec3::new(1, 0, -1),
    IVec3::new(-1, 0, -1),
];

pub const OMNI: [IVec3; 8] = [
    IVec3::X, 
    IVec3::new(-1,0,0), 
    IVec3::Z, 
    IVec3::new(0,0,-1),
    IVec3::new(1, 0, 1),
    IVec3::new(-1, 0, 1),
    IVec3::new(1, 0, -1),
    IVec3::new(-1, 0, -1),
];

pub const KNIGHT: [IVec3; 8] = [
    IVec3::new(2,0,1), IVec3::new(2,0,-1), IVec3::new(-2,0,1), IVec3::new(-2,0,-1),
    IVec3::new(1,0,2), IVec3::new(1,0,-2), IVec3::new(-1,0,2), IVec3::new(-1,0,-2),
];

pub const fn manhattan(a: IVec3, b: IVec3) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
}

pub const fn knight(a: IVec3, b: IVec3) -> i32 {
    (manhattan(a, b) + 2) / 3
}

pub const fn chebyshev(a: IVec3, b: IVec3) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    let dz = (a.z - b.z).abs();

    if dx >= dy && dx >= dz {
        dx
    } else if dy >= dz {
        dy
    } else {
        dz
    }
}

pub const STANDARD_MOVE : MoveRule = MoveRule {
    offsets: &STANDARD,
    can_climb: true,
    heuristic: manhattan,
};

pub const KNIGHT_MOVE : MoveRule = MoveRule {
    offsets: &KNIGHT,
    can_climb: false,
    heuristic: knight,
};

pub const ROOK_RULE: MoveRule = MoveRule {
    offsets: &STANDARD,
    can_climb: false,
    heuristic: manhattan,
};

pub const BISHOP_RULE: MoveRule = MoveRule {
    offsets: &DIAGONAL,
    can_climb: false,
    heuristic: chebyshev,
};


pub const QUEEN_RULE: MoveRule = MoveRule {
    offsets: &OMNI,
    can_climb: false,
    heuristic: chebyshev,
};