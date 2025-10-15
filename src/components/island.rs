use bevy::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use crate::components::island_maps::Map;

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

#[derive(Component)]
pub struct GenerateIsland;

#[derive(Component)]
pub struct MapFinishedIsland;

#[derive(Component)]
pub struct VisualizeIsland;

#[derive(Component)]
pub struct FinishedSetupIsland;

#[derive(Component)]
pub struct Waiting;

#[derive(Component)]
pub struct EliminationObjective;

#[derive(Component)]
pub struct CompletedIslandObjective;

#[derive(Component)]
pub struct Chest;

#[derive(Component)]
pub struct EnterPosition(pub Vec3);

pub trait IslandGen {
    fn generate_tiles(map: &mut Map, seed: u64, rng: &mut StdRng);
}