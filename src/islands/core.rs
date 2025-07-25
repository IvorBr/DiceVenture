use bevy::{prelude::*};
use rand::{rngs::StdRng, seq::SliceRandom};
use std::collections::HashSet;

use crate::components::island_maps::{Map, TerrainType, Tile, TileType};

pub fn reserve_with_margin(set: &mut HashSet<IVec3>, positions: &[IVec3], margin: i32) {
    for &pos in positions {
        for dx in -margin..=margin {
            for dy in -margin..=margin {
                for dz in -margin..=margin {
                    set.insert(pos + IVec3::new(dx, dy, dz));
                }
            }
        }
    }
}

pub fn add_boardwalk(
    map: &mut Map,
    reserved_positions: &mut HashSet<IVec3>,
    generator: &mut StdRng,
) {
    let mut shoreline_tiles = map.shore_tiles();
    shoreline_tiles.shuffle(generator);

    for board_location in shoreline_tiles {
        let directions = [
            -IVec3::Z, // North
            IVec3::Z,  // South
            IVec3::X,  // East
            -IVec3::X, // West
        ];  

        for dir in directions {
            if let Some(positions) = map.check_fit_rect(board_location, dir, 2, 5) {
                for pos in &positions {
                    map.add_entity_ivec3(
                        *pos,
                        Tile::new(TileType::Terrain(TerrainType::Boardwalk), Entity::PLACEHOLDER),
                    );
                    map.leave_position = *pos;
                }
                reserve_with_margin(reserved_positions, &positions, 1);
                return;
            }
        }
    }
}
