use bevy::{ecs::entity::MapEntities, prelude::*};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use crate::components::island_maps::Grid;

#[derive(Component)]
pub struct EleminationObjective;

#[derive(Component)]
pub struct IslandRoot;

#[derive(Debug, Deserialize, Message, Serialize, MapEntities, Clone)]
pub struct EnteredIsland(#[entities] pub Entity);

#[derive(Debug, Deserialize, Message, Serialize)]
pub struct LeaveIsland;

#[derive(Component)]
pub struct LeavePosition;

#[derive(Event)]
pub struct CleanIsland;

#[derive(Component, Deserialize, Serialize)]
pub struct OnIsland(#[entities] pub Entity);

#[derive(Component)]
pub struct GenerateIsland;

#[derive(Component)]
pub struct MapFinishedIsland;

#[derive(Component)]
pub struct VisualizeIsland;

#[derive(Component)]
pub struct FinishedSetupIsland;

#[derive(Component, Deserialize, Serialize)]
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
    fn generate_tiles(map: &mut Grid, seed: u64, rng: &mut StdRng);
}