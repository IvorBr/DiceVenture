use bevy::{prelude::*};
use rand::{rngs::StdRng, seq::SliceRandom};
use std::collections::HashSet;
use rand::Rng;
use crate::components::island_maps::{Map, TerrainType, Tile, TileType};
use rand::seq::IndexedRandom;
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

fn generate_trees(
    map: &mut Map,
    reserved_positions: &mut HashSet<IVec3>,
    generator: &mut StdRng,
    top_tiles: Vec<IVec3>
) {
    let num_trees = generator.random_range(3..=6);
    for _ in 0..num_trees {
        if let Some(&base) = top_tiles.choose(generator) {
            if reserved_positions.contains(&base) { continue; }

            let height = generator.random_range(2..=4);
            let mut tree_positions = vec![];

            // Trunk
            for i in 0..height {
                tree_positions.push(base + IVec3::new(0, i, 0));
            }

            // Leaves
            let leaf_size = generator.random_range(2..=3);
            let leaf_min = -(leaf_size / 2);
            let leaf_max = leaf_min + leaf_size - 1;

            for dx in leaf_min..=leaf_max {
                for dy in 0..=1 {
                    for dz in leaf_min..=leaf_max {
                        tree_positions.push(base + IVec3::new(dx, height + dy, dz));
                    }
                }
            }

            if tree_positions.iter().any(|p| reserved_positions.contains(p)) {
                continue;
            }
            
            for pos in tree_positions.iter() {
                let terrain = if pos.y < base.y + height {
                    TerrainType::TreeTrunk
                } else {
                    TerrainType::Leaves
                };

                map.add_entity_ivec3(
                    *pos,
                    Tile::new(TileType::Terrain(terrain), Entity::PLACEHOLDER),
                );
            }

            reserve_with_margin(reserved_positions, &tree_positions, 1);
        }
    }
}

