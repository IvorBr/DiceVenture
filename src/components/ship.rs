use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;

#[derive(Component, Serialize, Deserialize, Debug)]
#[require(Replicated)]
pub struct Ship;

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct PatrolIsland(pub Entity);