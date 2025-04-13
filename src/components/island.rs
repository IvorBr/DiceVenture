use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IslandType {
    #[default]
    Atoll,
    Forest,
    Grass,
    Desert,
    Ice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IslandObjective {
    #[default]
    Eliminate,
    Capture
}

#[derive(Component, Default)]
pub struct IslandInfo {
    pub island_type: IslandType,
    pub island_objective: IslandObjective
}

#[derive(Component)]
pub struct EleminationObjective;

#[derive(Component)]
pub struct IslandRoot;

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct EnteredIsland(pub u64);

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct LeaveIsland;

#[derive(Component)]
pub struct LeavePosition;

#[derive(Event)]
pub struct CleanIsland;

#[derive(Component, Serialize, Deserialize)]
pub struct OnIsland(pub u64);
