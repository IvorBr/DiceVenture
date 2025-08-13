use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use std::collections::HashMap;
use crate::plugins::attack::AttackId;
use bitflags::bitflags;

#[derive(Component, Serialize, Deserialize)]
pub struct Health{
    pub value : u64,
    pub max: u64
}

impl Health {
    pub fn new(value: u64) -> Self {
        Health { value: value , max: value}
    }

    pub fn get(&self) -> u64 {
        self.value
    }

    pub fn damage(&mut self, amount: u64) -> u64 {
        self.value = self.value.saturating_sub(amount);
        self.value
    }
}

impl Default for Health {
    fn default() -> Self {
        Self { value : 30, max: 30 }
    }
}

#[derive(Component, Serialize, Deserialize, Default)]
pub struct Position(pub IVec3);

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct MoveDirection(pub IVec3);

#[derive(Component, Serialize, Deserialize)]
pub struct RemoveEntity;

#[derive(Component, PartialEq, Eq, Default)]
pub enum ActionState {
    #[default]
    Idle,
    Moving,
    Attacking,
    Stunned,
}

#[derive(Component, Serialize, Deserialize, Default)]
#[require(Position)]
#[require(Health)]
#[require(ActionState)]
#[require(AttackCooldowns)]
#[require(StatusFlags)]
#[require(ActiveSkills)]
pub struct Humanoid;

#[derive(Component, Default)]
pub struct AttackCooldowns(pub HashMap<AttackId, Timer>);

bitflags! {
    #[derive(Default)]
    pub struct Status: u8 {
        const STUNNED = 0b00000001;
        const ROOTED =  0b00000010;
    }
}

#[derive(Component, Default)]
pub struct StatusFlags(pub Status);

#[derive(Component)]
pub struct Stunned {
    pub timer: Timer,
}

impl Stunned {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

#[derive(Component, Default)]
pub struct ActiveSkills(pub HashMap<u64, Entity>);

#[derive(Component)]
pub struct DamageVisualizer {
    pub timer: Timer,
    pub flash_color: Color,
    pub original_color: Option<Color>,
}

#[derive(Component, Deref, DerefMut)]
pub struct VisualRef(pub Entity);

#[derive(Component)]
#[require(Transform)]
pub struct VisualEntity;