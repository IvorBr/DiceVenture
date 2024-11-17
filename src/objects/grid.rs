
use bevy::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::CHUNK_SIZE;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Copy, Clone)]
pub enum TileType {
    #[default]
    Empty,
    Terrain,
    Player,
    Enemy,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Tile {
    pub kind : TileType,
    pub entity : Entity,
    pub parent_entity: Option<Entity>,
}

impl Tile {
    pub fn new(kind: TileType, entity: Entity) -> Self {
        Self {
            kind,
            entity,
            parent_entity: None,
        }
    }

    pub fn new_multi(kind: TileType, parent_entity: Entity) -> Self {
        Self {
            kind,
            entity: Entity::PLACEHOLDER,
            parent_entity: Some(parent_entity),
        }
    }
    
    pub fn reset(&mut self) {
        self.kind = TileType::Empty;
        self.entity = Entity::PLACEHOLDER;
        self.parent_entity = None;
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::new(TileType::Empty, Entity::PLACEHOLDER)
    }
}

pub struct Chunk {
    pub tiles: [Tile; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]      
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            tiles: [Tile::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize],
        }
    }

    fn index(&self, local_pos: IVec3) -> usize {
        (local_pos.x + CHUNK_SIZE * (local_pos.y + CHUNK_SIZE * local_pos.z)) as usize
    }

    pub fn get_tile(&self, local_pos: IVec3) -> Tile {
        self.tiles[self.index(local_pos)]
    }

    pub fn set_tile(&mut self, local_pos: IVec3, tile: Tile) {
        self.tiles[self.index(local_pos)] = tile;
    }

    pub fn reset_tile(&mut self, local_pos: IVec3) {
        self.tiles[self.index(local_pos)].reset();
    }
}
#[derive(Resource)]
pub struct Map {
    pub chunks: HashMap<IVec3, Chunk>,
}

impl Map {
    pub fn new() -> Self {
        let chunks = HashMap::new();
        Map { chunks }
    }

    pub fn world_to_chunk_coords(&self, world_pos: IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.div_euclid(CHUNK_SIZE),
            world_pos.y.div_euclid(CHUNK_SIZE),
            world_pos.z.div_euclid(CHUNK_SIZE),
        )
    }

    // Convert world position to local chunk coordinates
    pub fn world_to_local_chunk_coords(&self, world_pos: IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.rem_euclid(CHUNK_SIZE),
            world_pos.y.rem_euclid(CHUNK_SIZE),
            world_pos.z.rem_euclid(CHUNK_SIZE),
        )
    }

    // Get the chunk containing a given world position
    pub fn get_chunk(&self, world_pos: IVec3) -> Option<&Chunk> {
        let chunk_coords = self.world_to_chunk_coords(world_pos);
        self.chunks.get(&chunk_coords)
    }

    pub fn get_chunk_mut(&mut self, position: IVec3) -> Option<&mut Chunk> {
        let chunk_coords = self.world_to_chunk_coords(position);
        self.chunks.get_mut(&chunk_coords)
    }

    // Get or create the chunk at the given world position
    pub fn get_or_create_chunk(&mut self, world_pos: IVec3) -> &mut Chunk {
        let chunk_coords = self.world_to_chunk_coords(world_pos);
        self.chunks.entry(chunk_coords).or_insert_with(Chunk::new)
    }

    // Check if a tile is in the terrain or if it's empty (for movement)
    pub fn can_move(&self, position: IVec3) -> bool {
        if self.get_tile(position - IVec3::new(0, 1, 0)).kind == TileType::Terrain {
            return match self.get_tile(position).kind {
                TileType::Empty => true,
                TileType::Player => true,
                _ => false,
            };
        }
        false
    }

    // Get a tile at the world position
    pub fn get_tile(&self, position: IVec3) -> Tile {
        if let Some(chunk) = self.get_chunk(position) {
            return chunk.get_tile(self.world_to_local_chunk_coords(position))
        }
        Tile::default()
    }

    // Add an entity at the world position
    pub fn add_entity(&mut self, x: i32, y: i32, z: i32, tile: Tile) {
        let world_pos = IVec3::new(x, y, z);
        let local_coords = self.world_to_local_chunk_coords(world_pos);
        let chunk = self.get_or_create_chunk(world_pos);
        chunk.set_tile(local_coords, tile);
    }

    pub fn add_entity_ivec3(&mut self, position: IVec3, tile_type: Tile) {
        let local_coords = self.world_to_local_chunk_coords(position);
        let chunk = self.get_or_create_chunk(position);
        chunk.set_tile(local_coords, tile_type);
    }

    pub fn remove_entity(&mut self, position: IVec3) {
        let local_coords = self.world_to_local_chunk_coords(position);
        if let Some(chunk) = self.get_chunk_mut(position) {
            chunk.reset_tile(local_coords);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Copy, Clone)]
pub enum UpdateType {
    LoadTerrain,
    UnloadTerrain,
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct MapUpdate(pub UpdateType, pub IVec3, pub u32);

