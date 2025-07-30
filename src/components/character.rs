use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;

use crate::components::humanoid::Humanoid;

#[derive(Component, Serialize, Deserialize, Debug)]
#[require(Humanoid)]
#[require(Replicated)]
pub struct Character;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Resource)]
pub struct MovementCooldown {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PendingSkillCast {
    pub attack_id: u64,
}