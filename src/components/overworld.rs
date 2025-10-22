use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::components::{core::Seed, island_maps::Grid};

#[derive(Component)]
pub struct Ocean;

#[derive(Component, Default, Deserialize, Serialize)]
#[require(Enterable, Replicated)]
pub struct Island {
    pub position: Vec3
}

#[derive(Component,Deserialize, Serialize)]
#[require(Island)]
pub struct StarterIsland;

#[derive(Component)]
pub struct LocalIsland;

#[derive(Component)]
pub struct ProximityUI;

#[derive(Component)]
pub struct OverworldUI;

#[derive(Component)]
pub struct OverworldIslandVisual;

#[derive(Component)]
pub struct OverworldRoot;

#[derive(Component, Default)]
#[require(Grid, Seed)]
pub struct Enterable;

#[derive(Component)]
pub struct Entered;

#[derive(Deserialize, Event, Serialize, MapEntities, Clone)]
pub struct ClientShipPosition{
    pub position: Vec3,
    #[entities]
    pub entity: Entity
}

#[derive(MapEntities, Deserialize, Event, Serialize)]
pub struct ServerShipPosition{
    pub position: Vec3,
    #[entities]
    pub entity: Entity
}

#[derive(Resource, Clone, Copy)]
pub struct WorldSeed(pub u64);