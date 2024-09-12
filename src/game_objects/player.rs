use bevy::prelude::*;
use crate::game_objects::humanoid::{Health, Position};
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::{Replicated, ClientId};

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    position: Position,
    health: Health,
    replicated: Replicated,
}

impl PlayerBundle {
    pub fn new(client_id: ClientId, health: u128, pos_vec: IVec3) -> Self {
        Self {
            player: Player(client_id),
            position: Position(pos_vec),
            health: Health::new(health),
            replicated: Replicated,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct Player(pub ClientId);