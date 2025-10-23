use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::shader::ShaderRef;
use crate::preludes::network_preludes::RenetServer;
use bevy_replicon::prelude::AppRuleExt;
use crate::components::core::{Initialized, Seed};
use crate::components::island::OnIsland;
use crate::components::character::LocalPlayer;
use crate::components::ship::Ship;
use crate::islands::atoll::Atoll;
use crate::GameState;
use crate::components::overworld::*;
use crate::plugins::camera::{CameraTarget, NewCameraTarget, LAYER_WATER};
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(ShaderType, Clone, Copy, Default)]
pub struct WaterUniform {
    pub camera_state: i32,
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub reflection: WaterUniform,
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            reflection: WaterUniform::default(),
        }
    }
}

impl Material for WaterMaterial {
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
    fn fragment_shader() -> ShaderRef { "shaders/water.wgsl".into() }
    fn vertex_shader() -> ShaderRef { "shaders/water.wgsl".into() }
}

pub const WATER_HEIGHT : f32 = 0.3;
pub struct OverworldPlugin;
impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app
        .replicate::<Island>()
        .replicate::<OnIsland>()
        .replicate::<StarterIsland>()
        .add_plugins((MeshPickingPlugin, MaterialPlugin::<WaterMaterial>::default()))
        .add_systems(OnEnter(GameState::Overworld), (activate_overworld, spawn_overworld_ui))
        .add_systems(OnExit(GameState::Overworld), overworld_cleanup)
        .add_systems(Startup, (setup_overworld, generate_islands.run_if(resource_exists::<RenetServer>)).chain())
        .add_systems(
            Update,
            (
                move_ocean,
                spawn_island
            )
        );
    }
}

fn spawn_island(
    mut commands: Commands,
    new_islands: Query<(Entity, &Island), Added<Island>>,
    overworld_query: Query<Entity, With<OverworldRoot>>,
){
    if let Ok(overworld_root) = overworld_query.single() {
        for (island_entity, island) in new_islands.iter() {
            commands.entity(island_entity)
            .insert((
                Transform::from_translation(island.position),
                Visibility::Inherited,
                Atoll,
                Initialized
            ))
            .insert(ChildOf(overworld_root));
        }
    }
}

fn overworld_cleanup(
    mut commands: Commands,
    ui_query: Query<Entity, With<OverworldUI>>,
    mut overworld_query: Query<(&ChildOf, &mut Visibility), With<OverworldIslandVisual>>,
    local_island: Query<Entity, With<LocalIsland>>
) {
    if let Ok(ui_entity) = ui_query.single() {
        commands.entity(ui_entity).despawn();
    }

    for (parent, mut overworld_visiblity) in overworld_query.iter_mut() {
        if local_island.get(parent.0).is_ok() {
            *overworld_visiblity = Visibility::Hidden;
        }
    //    *overworld_visiblity = Visibility::Hidden; //TODO: maybe its cooler to keep these in the map instead of making them dissapear
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
    mut overworld_query: Query<&mut Visibility, With<OverworldIslandVisual>>,
    ship_query: Query<Entity, (With<Ship>, With<LocalPlayer>)>,
    island_query: Query<Entity, With<LocalIsland>>
) {
    for mut overworld_visiblity in overworld_query.iter_mut() {
        *overworld_visiblity = Visibility::Visible;
    }

    if let Ok(ship_entity) = ship_query.single() {
        commands.entity(ship_entity).insert(NewCameraTarget);
    }

    if let Ok(local_island) = island_query.single() {
        commands.entity(local_island).remove::<LocalIsland>();
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

fn generate_islands(
    mut commands: Commands,
    overworld_query: Query<Entity, With<OverworldRoot>>,
    world_seed: Res<WorldSeed>
){
    if let Ok(overworld_root) = overworld_query.single() {
        //starter island
        commands
        .spawn((
            StarterIsland,
            Island{position: Vec3::ZERO},
            Seed(0)
        ))
        .insert(ChildOf(overworld_root));

        let mut rng = ChaCha8Rng::seed_from_u64(world_seed.0);
        let positions = poisson_disk_sample_2d(
            Vec2::ZERO,
            5.0 * 8.0,     // min distance between islands
            20,      // number of islands
            30.0 * 8.0,    // spread range
            &mut rng,
        );

        for (i, pos) in positions.into_iter().enumerate() {
            commands
            .spawn((
                Island { position: Vec3::new(pos.x.round(), 0.0, pos.y.round()) },
                Seed((i + 1) as u64)
            ))
            .insert(ChildOf(overworld_root));
        }
    }
}

fn setup_overworld(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    println!("SETUP WORLD");
    commands
    .spawn((
        OverworldRoot,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Visible,
    ));

    // ocean
    commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(75.0, 75.0).subdivisions(500))),
            MeshMaterial3d(water_materials.add(WaterMaterial {
                ..Default::default()
            })),
            Transform::from_xyz(0.0, WATER_HEIGHT, 0.0),
            Ocean,
            RenderLayers::layer(LAYER_WATER.into()),
        ));
}

fn move_ocean(
    mut ocean: Query<&mut Transform, With<Ocean>>,
    target: Query<&GlobalTransform, (With<CameraTarget>, Without<Ocean>)>
){
    if let Ok(transform) = target.single() {
        for mut ocean_transform in &mut ocean {
            ocean_transform.translation.x = transform.translation().x;
            ocean_transform.translation.z = transform.translation().z;
        }
    }
}