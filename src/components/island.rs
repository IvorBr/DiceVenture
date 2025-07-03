use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct EleminationObjective;

#[derive(Component)]
pub struct IslandRoot;

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct EnteredIsland(pub u64);

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct LeaveIsland(pub u64);

#[derive(Component)]
pub struct LeavePosition;

#[derive(Event)]
pub struct CleanIsland;

#[derive(Component, Serialize, Deserialize)]
pub struct OnIsland(pub u64);
