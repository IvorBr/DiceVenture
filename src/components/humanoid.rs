use bevy_replicon::core::ClientId;
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

impl Default for Health {
    fn default() -> Self {
        Self { value : 100 }
    }
}

#[derive(Component, Serialize, Deserialize, Default)]
pub struct Position(pub IVec3);

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct MoveDirection(pub IVec3);

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct AttackDirection(pub IVec3);

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
pub struct Humanoid;

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct AttackAnimation {
    pub client_id: ClientId,
    pub direction: IVec3,
}

#[derive(Component)]
pub struct AttackLerp {
    pub direction: IVec3,
    pub timer: Timer,
}