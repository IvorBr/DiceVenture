use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::island::IslandInfo;
use bevy_replicon::{core::ClientId, prelude::Replicated};

#[derive(Component, Serialize, Deserialize, Debug)]
#[require(Replicated)]
pub struct Ship(pub ClientId);

#[derive(Component)]
pub struct Ocean;

#[derive(Component, Default)]
#[require(IslandInfo)]
pub struct Island;

#[derive(Component)]
#[require(Island)]
pub struct StarterIsland;

#[derive(Component)]
pub struct SelectedIsland;

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
    pub client_id: ClientId,
    pub position: Vec3
}