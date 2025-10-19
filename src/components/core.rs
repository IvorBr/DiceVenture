use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Initialized;

#[derive(Component, Deserialize, Serialize, Default)]
pub struct Seed(pub u64);

