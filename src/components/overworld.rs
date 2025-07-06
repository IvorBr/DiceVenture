use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_replicon::prelude::Replicated;

#[derive(Component, Serialize, Deserialize, Debug)]
#[require(Replicated)]
pub struct Ship;

#[derive(Component)]
pub struct Ocean;

#[derive(Component, Default)]
pub struct Island(pub u64);

#[derive(Component)]
#[require(Island)]
pub struct StarterIsland;

#[derive(Component)]
pub struct LocalIsland;

#[derive(Component)]
pub struct ProximityUI;

#[derive(Component)]
pub struct OverworldUI;

#[derive(Component)]
pub struct OverworldRoot;

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct ClientShipPosition(pub Vec3);

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct ServerShipPosition{
    pub client_entity: Entity,
    pub position: Vec3
}