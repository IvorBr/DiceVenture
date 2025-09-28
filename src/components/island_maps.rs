
use bevy::{platform::collections::HashSet, prelude::*};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::CHUNK_SIZE;

#[derive(Resource, Default)]
pub struct IslandMaps {
    pub maps: HashMap<u64, Map>,
}

impl IslandMaps {
    pub fn new() -> Self {
        IslandMaps { maps: HashMap::new() }
    }

    pub fn get_map_mut(&mut self, id: u64) -> Option<&mut Map> {
        self.maps.get_mut(&id)
    }

    pub fn get_map(&self, id: u64) -> Option<&Map> {
        self.maps.get(&id)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Copy, Clone)]
pub enum TerrainType {
    Invisible,
    Sand,
    Rock,
    Boardwalk,
    PalmTree,
    TreeTrunk,
    Leaves
}

pub fn is_base_terrain(tile: &TerrainType) -> bool {
    matches!(tile, TerrainType::Sand | TerrainType::Rock)
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Copy, Clone)]
pub enum TileType {
    #[default]
    Empty,
    Terrain(TerrainType),
    Player,
    Enemy,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Tile {
    pub kind : TileType,
    pub entity : Entity,
}

impl Tile {
    pub fn new(kind: TileType, entity: Entity) -> Self {
        Self {
            kind,
            entity,
        }
    }
    
    pub fn reset(&mut self) {
        self.kind = TileType::Empty;
        self.entity = Entity::PLACEHOLDER;
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

pub struct Map {
    pub chunks: HashMap<IVec3, Chunk>,
    pub player_count : u32,
    pub enemy_count : u32,
    pub leave_position : IVec3,
    pub entities: HashSet<Entity>
}

impl Map {
    pub fn new() -> Self {
        let chunks = HashMap::new();
        let player_count = 0;
        let enemy_count = 0;
        let leave_position = IVec3::ZERO;
        let entities = HashSet::new();

        Map { chunks, player_count, enemy_count, leave_position, entities }
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

    pub fn chunk_to_world_coords(&self, chunk_coords: IVec3, tile_index: usize) -> IVec3 {
        let size = CHUNK_SIZE as usize;

        let x = (tile_index % size) as i32;
        let y = ((tile_index / size) % size) as i32;
        let z = (tile_index / (size * size)) as i32;

        chunk_coords * CHUNK_SIZE + IVec3::new(x, y, z)    
    }

    pub fn reset(&mut self) {
        self.chunks.clear();
        self.player_count = 0;
        self.leave_position = IVec3::ZERO;
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
        let below = position - IVec3::Y;
        if !matches!(self.get_tile(below).kind, TileType::Terrain(_)) {
            return false;
        }

        match self.get_tile(position).kind {
            TileType::Empty | TileType::Player => true,
            _ => false,
        }
    }

    // Get a tile at the world position
    pub fn get_tile(&self, position: IVec3) -> Tile {
        if let Some(chunk) = self.get_chunk(position) {
            return chunk.get_tile(self.world_to_local_chunk_coords(position))
        }
        Tile::default()
    }

    pub fn get_target(&self, position: IVec3) -> Option<Entity> {
        if let Some(chunk) = self.get_chunk(position) {
            let tile = chunk.get_tile(self.world_to_local_chunk_coords(position));
            if tile.kind == TileType::Enemy || tile.kind == TileType::Player {
                return Some(tile.entity)
            }
        }
        None
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

    pub fn add_player(&mut self, position: IVec3, entity: Entity){
        self.player_count += 1;
        self.add_entity_ivec3(position, Tile::new(TileType::Player, entity));
        self.entities.insert(entity);
    }

    pub fn add_enemy(&mut self, position: IVec3, entity: Entity){
        self.enemy_count += 1;
        self.add_entity_ivec3(position, Tile::new(TileType::Enemy, entity));
        self.entities.insert(entity);
    }

    pub fn update_position(&mut self, entity: Entity, position: IVec3, tile_type: TileType) {
        self.remove_entity(position);
        self.add_entity_ivec3(position, Tile::new(tile_type, entity));
    }

    pub fn shore_tiles(&mut self) -> Vec<IVec3> {
        let neighbors = [
            IVec3::X,
            -IVec3::X,
            IVec3::Z,
            -IVec3::Z,
        ];
        let mut chunk_entries: Vec<_> = self.chunks.iter().collect();
        chunk_entries.sort_by_key(|(key, _)| (key.x, key.y, key.z));
        let mut shore_tiles = Vec::new();

        for (chunk_coords, chunk) in chunk_entries {
            for (i, tile) in chunk.tiles.iter().enumerate() {
                if tile.kind == TileType::Terrain(TerrainType::Sand) {
                    let world_pos = self.chunk_to_world_coords(*chunk_coords, i);

                    if world_pos.y != 0 {
                        continue;
                    }

                    let is_shore = neighbors.iter().any(|&offset| {
                        let neighbor = world_pos + offset;
                        self.get_tile(neighbor).kind == TileType::Empty
                    });

                    if is_shore {
                        shore_tiles.push(world_pos);
                    }
                }
            }
        }

        shore_tiles
    }

    pub fn above_water_top_tiles(&self) -> Vec<IVec3> {
        let mut chunk_entries: Vec<_> = self.chunks.iter().collect();
        chunk_entries.sort_by_key(|(key, _)| (key.x, key.y, key.z));

        let mut result: Vec<IVec3> = Vec::new();
        for (chunk_coords, chunk) in chunk_entries {
            for (i, tile) in chunk.tiles.iter().enumerate() {
                if let TileType::Terrain(terrain_type) = tile.kind {
                    let pos = self.chunk_to_world_coords(*chunk_coords, i);
                    if pos.y < 0 || !is_base_terrain(&terrain_type) {
                        continue;
                    }

                    let key = (pos.x, pos.z);
                    match result.iter_mut().find(|v| (v.x, v.z) == key) {
                        Some(existing) => {
                            if pos.y > existing.y {
                                *existing = pos;
                            }
                        }
                        None => {
                            result.push(pos);
                        }
                    }
                }
            }
        }

        result
    }

    pub fn check_fit_rect(
        &self,
        origin: IVec3,
        direction: IVec3,
        width: i32,
        length: i32,
    ) -> Option<Vec<IVec3>> {
            let right = IVec3::new(-direction.z, 0, direction.x);
            let mut positions = Vec::with_capacity((width * length) as usize);

            for l in 0..length {
                for w in 0..width {
                    let pos = origin + direction * l + right * w;

                    let tile = self.get_tile(pos).kind;
                    let is_first_row = l == 0;

                    let acceptable = is_first_row || tile == TileType::Empty;

                    if !acceptable {
                        return None;
                    }

                    positions.push(pos);
                }
            }

            Some(positions)
        }
    }

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Copy, Clone)]
pub enum UpdateType {
    LoadTerrain,
    UnloadTerrain,
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct MapUpdate(pub UpdateType, pub IVec3, pub u32);

