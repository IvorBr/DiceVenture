use bevy::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component, Default)]
pub struct Island;

#[derive(Component)]
#[require(Island)]
pub struct StarterIsland;




