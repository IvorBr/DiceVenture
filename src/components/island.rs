use bevy::prelude::*;

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