use bevy::prelude::*;
use crate::components::island_maps::{Map, TileType};

pub fn check_attack_path(
    start: IVec3,
    direction: IVec3,
    range: u8,
    map: &Map
) -> Vec3 {
    let mut vector = Vec3::ZERO;
    let mut total_down = 0;

    for i in 1..=range {
        let tile_pos = start + direction * i as i32 + IVec3::new(0, -total_down, 0);
        let tile = map.get_tile(tile_pos);
        
        match tile.kind {
            TileType::Terrain(_) => {
                if total_down == 0 {
                    vector = Vec3::new(0.0, 1.0/(i*2 - 1) as f32, 0.0);
                    break;
                }
                else{
                    vector = Vec3::ZERO;
                    break;
                }
            },
            TileType::Empty => {
                let new_tile = map.get_tile(tile_pos + IVec3::new(0,-1,0));

                if new_tile.kind == TileType::Empty {
                    if total_down == i as i32 - 1 {
                        total_down += 1;
                        if total_down >= range as i32/4 {
                            vector = -Vec3::Y;
                            break;
                        }
                    }
                }
                else if total_down == 1 {
                    vector = Vec3::new(0.0, -0.5, 0.0);
                    break;
                }
                //else: the path is straight
            },
            _ => (),
        }
    }

    direction.as_vec3() + vector
}