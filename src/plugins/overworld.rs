use bevy::prelude::*;
use crate::components::island::{EnteredIsland, GenerateIsland, VisualizeIsland};
use crate::components::character::LocalPlayer;
use crate::islands::atoll::Atoll;
use crate::GameState;
use crate::components::overworld::*;
use crate::plugins::camera::{CameraTarget, NewCameraTarget};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    pub random_number: i32,
    #[texture(1)]
    #[sampler(2)]
    pub depth_texture: Handle<Image>,
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            random_number: 0,
            depth_texture: Handle::default()
        }
    }
}

impl Material for WaterMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/water_shader.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/water_shader.wgsl".into()
    }
}

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((MeshPickingPlugin, MaterialPlugin::<WaterMaterial>::default()))
        .add_systems(OnEnter(GameState::Overworld), (spawn_overworld, activate_overworld, spawn_overworld_ui))
        .add_systems(OnExit(GameState::Overworld), overworld_cleanup)
        .add_systems(
            Update,
            (
                move_ocean,
                island_proximity.run_if(in_state(GameState::Overworld)),
            )
        );
    }
}

fn overworld_cleanup(
    mut commands: Commands,
    ui_query: Query<Entity, With<OverworldUI>>,
    mut overworld_query: Query<&mut Visibility, With<OverworldRoot>>,
) {
    if let Ok(ui_entity) = ui_query.single() {
        commands.entity(ui_entity).despawn();
    }

    if let Ok(mut overworld_visiblity) = overworld_query.single_mut() {
        *overworld_visiblity = Visibility::Hidden;
    }
}

fn spawn_overworld_ui(
    mut commands: Commands
) {
    commands.spawn(
        (Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }, 
    OverworldUI
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Enter island - press F"),
            TextFont {
                font_size: 25.0,
                ..default()
            },
            ProximityUI,
            Visibility::Hidden
        ));
    });
}

fn activate_overworld(
    mut commands: Commands,
    mut overworld_query: Query<&mut Visibility, With<OverworldRoot>>,
    ship_query: Query<Entity, (With<Ship>, With<LocalPlayer>)>,
    island_query: Query<Entity, With<LocalIsland>>
) {
    if let Ok(mut overworld_visiblity) = overworld_query.single_mut() {
        *overworld_visiblity = Visibility::Visible;
        
        if let Ok(ship_entity) = ship_query.single() {
            commands.entity(ship_entity).insert(NewCameraTarget);
        }

        if let Ok(local_island) = island_query.single() {
            commands.entity(local_island).remove::<LocalIsland>();
        }
    }
}

fn poisson_disk_sample_2d(
    center: Vec2,
    radius: f32,
    num_points: usize,
    range: f32,
    rng: &mut impl rand::Rng,
) -> Vec<Vec2> {
    let mut points = Vec::new();
    points.push(Vec2 { x: 0.0, y: 0.0 });

    while points.len() < num_points {
        let candidate = Vec2::new(
            rng.random_range(-range..range),
            rng.random_range(-range..range),
        );

        if points.iter().all(|p : &Vec2| p.distance(candidate) >= radius) {
            points.push(center + candidate);
        }
    }

    points[1..].to_vec()
}

fn spawn_overworld(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    overworld_query: Query<&OverworldRoot>,
    world_seed: Res<WorldSeed>
) {
    println!("spawning {}", world_seed.0);
    if overworld_query.single().is_ok() {
        return;
    }

    let overworld_root = commands
        .spawn((
            OverworldRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Visible,
        ))
        .id();

    // ocean
    commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(500))),
            MeshMaterial3d(water_materials.add(WaterMaterial {
                ..Default::default()
            })),
            Transform::from_xyz(0.0, 0.3, 0.0),
            Ocean,
        ))
        .observe(on_clicked_ocean);

    // starter island
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(100, 255, 100),
                ..Default::default()
            })),
            Transform::from_xyz(0.0, 0.2, 0.0),
            StarterIsland,
            Island(0),
            Visibility::Inherited,
            Atoll
        ))
        .observe(on_clicked_island)
        .insert(ChildOf(overworld_root));

    let mut rng = ChaCha8Rng::seed_from_u64(world_seed.0);
    let positions = poisson_disk_sample_2d( //should end up basing this on a seed and chunk, since now we are only doing this in a small range
        Vec2::ZERO,
        5.0,     // min distance between islands
        20,      // number of islands
        30.0,    // spread range
        &mut rng,
    );

    for (i, pos) in positions.into_iter().enumerate() {
        let (island_type, base_color) = if rng.random_bool(0.5) {
            (Atoll,
            Color::srgb(0.9, 0.8, 0.6))
        } else {
            (Atoll,
            Color::srgb(0.0, 0.4, 0.0))
        };
        
        // Spawn the island entity
        commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                ..Default::default()
            })),
            Transform::from_xyz(pos.x, 0.2, pos.y),
            Visibility::Inherited, 
            Island((i + 1) as u64),
            Atoll
        ))
        .observe(on_clicked_island)
        .insert(ChildOf(overworld_root));
    }
}

fn move_ocean(
    mut ocean: Query<&mut Transform, With<Ocean>>,
    target: Query<&Transform, (With<CameraTarget>, Without<Ocean>)>
){
    if let Ok(transform) = target.single() {
        for mut ocean_transform in &mut ocean {
            ocean_transform.translation.x = transform.translation.x;
            ocean_transform.translation.z = transform.translation.z;
        }
    }
}

fn island_proximity_check(
    position: Transform,
    islands: Vec<(Transform, u64, Entity)> 
) -> Option<(u64, f32, Entity)> {
    if islands.is_empty() {
        return None;
    }

    let (island_transform, mut best_island, mut best_island_enitity) = islands[0];
    let mut best_distance = position.translation.distance_squared(island_transform.translation);

    for (transform, island_id, entity) in islands.iter().skip(1) {
        let distance = position.translation.distance_squared(transform.translation);
        if distance < best_distance {
            best_distance = distance;
            best_island = *island_id;
            best_island_enitity = *entity;
        }
    }

    Some((best_island, best_distance, best_island_enitity))
}

fn island_proximity(
    mut commands: Commands,
    mut proximity_ui_query: Query<&mut Visibility, With<ProximityUI>>,
    ship_query: Query<&Transform, (With<Ship>, With<LocalPlayer>)>,
    island_query: Query<(&Transform, &Island, Entity)>, //TODO: CURRENTLY DOES ALL ISLANDS!! SHOULD JUST BE THE CURRENT CHUNK
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut player_enter_event: EventWriter<EnteredIsland>
) {
    if let Ok(ship_transform) = ship_query.single() {
        let island_transforms: Vec<(Transform, u64, Entity)> = island_query.iter().map(|(t, i, e)| (*t, i.0, e)).collect();
        
        if let Some((island_id, closest_distance, island_entity)) = island_proximity_check(*ship_transform, island_transforms) {
            if let Ok(mut proximity_ui_visibility) = proximity_ui_query.single_mut() {
                if closest_distance < 2.0 {
                    *proximity_ui_visibility = Visibility::Inherited;

                    if keyboard_input.pressed(KeyCode::KeyF) {
                        commands.entity(island_entity).insert(LocalIsland).insert(GenerateIsland).insert(VisualizeIsland);
                        println!("Adding generate for island: {}, {}, {}", island_id, island_entity, closest_distance);
                        player_enter_event.write(EnteredIsland(island_id));
                        state.set(GameState::Island);
                    }
                } else {
                    *proximity_ui_visibility = Visibility::Hidden;
                }
            }

            
        }
    }
}

fn on_clicked_island(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands, 
    current_target: Query<Entity, With<CameraTarget>>,
    game_state: Res<State<GameState>>
) {
    println!("state: {:?}", *game_state.get());
    if *game_state.get() == GameState::Overworld {
        if let Ok(target) = current_target.single() {
            commands.entity(target).remove::<CameraTarget>();
        }
    
        commands.entity(click.target()).insert(CameraTarget);
    
        println!("Entity {:?} is now the camera target! island", click.target());
    }
}

fn on_clicked_ocean(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    current_target: Query<Entity, (With<CameraTarget>,Without<Ship>)>,
    ship: Query<Entity, (With<Ship>, Without<CameraTarget>)>,
    game_state: Res<State<GameState>>
) {
    println!("state: {:?}", *game_state.get());

    if *game_state.get() == GameState::Overworld {
        println!("Entity {:?} is now the camera target! ocean", click.target());

        if let Ok(target) = current_target.single() {
            commands.entity(target).remove::<CameraTarget>();
        }

        if let Ok(local_ship) = ship.single() {
            commands.entity(local_ship).insert(CameraTarget);
        }
    }
}
