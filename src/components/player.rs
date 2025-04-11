use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;

use crate::components::humanoid::Humanoid;

#[derive(Component, Serialize, Deserialize, Debug)]
#[require(Humanoid)]
#[require(Replicated)]
pub struct Player;

#[derive(Component)]
pub struct LocalPlayer;