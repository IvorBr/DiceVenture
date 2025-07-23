use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use std::collections::HashMap;
use crate::plugins::attack::AttackId;

#[derive(Component, Serialize, Deserialize)]
pub struct Health{
    value : u64,
}

impl Health {
    pub fn new(value: u64) -> Self {
        Health { value }
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
        Self { value : 30 }
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
}

#[derive(Component, Serialize, Deserialize, Default)]
#[require(Position)]
#[require(Health)]
#[require(ActionState)]
#[require(AttackCooldowns)]
pub struct Humanoid;

    #[derive(Component, Default)]
    pub struct AttackCooldowns(pub HashMap<AttackId, Timer>);