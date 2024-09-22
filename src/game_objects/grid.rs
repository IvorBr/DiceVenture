
use bevy::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Copy, Clone)]
pub enum Tile {
    #[default]
    Terrain,
    Player(Entity),
    Enemy(Entity),
}

#[derive(Resource)]
pub struct Map {
    pub grid: HashMap<IVec3, Tile>,
}

impl Map {
    pub fn new() -> Self {
        let grid = HashMap::new();
        Map { grid }
    }

    pub fn cell(&self, position : IVec3) -> Option<Tile> {
        self.grid.get(&position).cloned()
    }

    pub fn add_entity(&mut self, x: i32, y: i32, z: i32, tile_type : Tile) {
        self.grid.insert(IVec3::new(x, y, z), tile_type);
    }

    pub fn add_entity_ivec3(&mut self, position : IVec3, tile_type : Tile) {
        self.grid.insert(position, tile_type);
    }

    pub fn remove_entity(&mut self, position : IVec3) {
        self.grid.remove(&position);
    }
}

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct MapUpdate(pub IVec3, pub u32, pub Tile);

