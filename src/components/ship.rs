use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;

use crate::components::overworld::{Enterable, OverworldPosition};

#[derive(Component, Serialize, Deserialize, Debug)]
#[require(Replicated, Enterable, OverworldPosition)]
pub struct Ship;

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct PatrolIsland(pub Entity);