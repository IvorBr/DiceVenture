use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Component, Serialize, Deserialize)]
pub struct Health{
    value : u128,
}

impl Health {
    pub fn new(value: u128) -> Self {
        Health { value }
    }

    pub fn get(&self) -> u128 {
        self.value
    }

    pub fn damage(&mut self, amount: u128) {
        self.value = self.value.saturating_sub(amount);
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct Position(pub IVec3);

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct MoveDirection(pub IVec3);