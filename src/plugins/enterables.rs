use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{random_range, SeedableRng};
use crate::components::island::{EnterPosition, GenerateIsland, IslandGen, MapFinishedIsland};
use crate::components::character::LocalPlayer;
use crate::components::island_maps::{Grid, TerrainType, TileType};
use crate::components::ship::Ship;
use crate::GameState;
use crate::components::overworld::*;

pub struct EnterablesPlugin;
impl Plugin for EnterablesPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, enterable_proximity_check.run_if(in_state(GameState::Overworld)));
    }
}

fn find_closest_entity(
    position: Transform,
    entities: Vec<(Vec3, Entity)> 
) -> Option<(f32, Entity)> {
    if entities.is_empty() {
        return None;
    }

    let (first_position, mut best_enitity) = entities[0];
    let mut best_distance = position.translation.distance_squared(first_position);

    for (enter_position, entity) in entities.iter().skip(1) {
        let distance = position.translation.distance_squared(*enter_position);
        if distance < best_distance {
            best_distance = distance;
            best_enitity = *entity;
        }
    }

    Some((best_distance, best_enitity))
}

fn enterable_proximity_check(
    mut commands: Commands,
    mut proximity_ui_query: Query<&mut Visibility, With<ProximityUI>>,
    ship_query: Query<(Entity, &Transform), (With<Ship>, With<LocalPlayer>)>,
    entity_query: Query<(&EnterPosition, &Transform, Entity), With<Enterable>>, //TODO: CURRENTLY DOES ALL ENTERABLES!! SHOULD JUST BE THE CURRENT CHUNK or some defined subset
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((ship_entity, ship_transform)) = ship_query.single() {
        let entity_transforms: Vec<(Vec3, Entity)> = entity_query.iter().filter(|(_, _, en)| *en != ship_entity).map(|(ep, tf, en)| (ep.0 + tf.translation, en)).collect();
        
        if let Some((closest_distance, enterable_entity)) = find_closest_entity(*ship_transform, entity_transforms) {
            if let Ok(mut proximity_ui_visibility) = proximity_ui_query.single_mut() {
                if closest_distance < 4.0 {
                    *proximity_ui_visibility = Visibility::Inherited;

                    if keyboard_input.pressed(KeyCode::KeyF) {
                        commands.entity(enterable_entity).insert(Entered);
                    }
                } else {
                    *proximity_ui_visibility = Visibility::Hidden;
                }
            }

            
        }
    }
}

pub fn generate_island_overworld<T: Component + IslandGen>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    new_islands: Query<(Entity, &Island, &Transform), (Added<Enterable>, With<T>)>,
) {
    let island_size = 1.0;
    for (entity, island_id, tf) in new_islands.iter() {
        let mut generator = StdRng::seed_from_u64(island_id.seed);

        let mut map = Grid::new();
        T::generate_tiles(&mut map, island_id.seed, &mut generator);

        let parent = commands.spawn((OverworldIslandVisual, Transform::from_translation(Vec3::ZERO), Visibility::Visible)).id();
        commands.entity(entity).add_child(parent).insert(EnterPosition(map.leave_position.as_vec3()/island_size));

        for (chunk_pos, chunk) in map.chunks.iter() {
            for (tile_index, tile) in chunk.tiles.iter().enumerate() {
                let mut color: Color;
                match tile.kind {
                    TileType::Terrain(TerrainType::Sand) => {
                        color = Color::srgb(0.9, 0.8, 0.6);
                    }
                    TileType::Terrain(TerrainType::Rock) => {
                        color = Color::srgb(0.49, 0.51, 0.52);
                    }
                    TileType::Terrain(TerrainType::Boardwalk) => {
                        color = Color::srgb_u8(100, 60, 40);
                    }
                    TileType::Terrain(TerrainType::PalmTree) => {
                        color = Color::srgb_u8(88, 57, 39);

                        let child = commands.spawn((
                            Mesh3d(meshes.add(Cuboid::new(1.0/island_size, 1.0/island_size, 1.0/island_size))),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgb_u8(36, 80, 2),
                                ..Default::default()
                            })),
                            Transform::from_translation((map.chunk_to_world_coords(*chunk_pos, tile_index) + IVec3::Y).as_vec3()/island_size),
                            Visibility::Inherited
                        )).id();

                        commands.entity(parent).add_child(child);
                    }
                    TileType::Terrain(TerrainType::TreeTrunk) => {
                        color = Color::srgb_u8(88, 57, 39);
                    }
                    TileType::Terrain(TerrainType::Leaves) => {
                        color = Color::srgb_u8(36, 80, 2);
                    }
                    _ => continue,
                }
                
                let jitter = random_range(-0.05..=0.05);
                let [r0, g0, b0, a] = color.to_linear().to_f32_array();
                let r = (r0 + jitter).clamp(0.0, 1.0);
                let g = (g0 + jitter).clamp(0.0, 1.0);
                let b = (b0 + jitter).clamp(0.0, 1.0);

                color = Color::srgba(r, g, b, a);
                let child = commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0/island_size, 1.0/island_size, 1.0/island_size))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        ..Default::default()
                    })),
                    Transform::from_translation((map.chunk_to_world_coords(*chunk_pos, tile_index)).as_vec3()/island_size),
                    Visibility::Inherited
                )).id();

                commands.entity(parent).add_child(child);
            }
            
        }
    }
}

pub fn generate_island_map<T: Component + IslandGen>(
    mut commands: Commands,
    mut new_islands: Query<(Entity, &Island, &mut Grid), (With<T>, With<GenerateIsland>)>
) {
    for (entity, island_id, mut grid) in new_islands.iter_mut() {
        let mut generator = StdRng::seed_from_u64(island_id.seed);

        T::generate_tiles(&mut grid, island_id.seed, &mut generator);
        commands.entity(entity).insert(MapFinishedIsland).remove::<GenerateIsland>();
    }
}