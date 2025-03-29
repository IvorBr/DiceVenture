use bevy::prelude::*;

use crate::components::island::IslandInfo;

#[derive(Component)]
pub struct Ship;

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

