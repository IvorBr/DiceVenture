use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;

use crate::components::overworld::Enterable;

#[derive(Component, Serialize, Deserialize, Debug)]
#[require(Replicated, Enterable)]
pub struct Ship;

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct PatrolIsland(pub Entity);