use bevy::prelude::*;
use crate::components::island::EnteredIsland;
use crate::components::player::LocalPlayer;
use crate::GameState;
use crate::components::overworld::*;
use crate::plugins::camera::{CameraTarget, NewCameraTarget};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

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
    if let Ok(ui_entity) = ui_query.get_single() {
        commands.entity(ui_entity).despawn_recursive();
    }

    if let Ok(mut overworld_visiblity) = overworld_query.get_single_mut() {
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
    if let Ok(mut overworld_visiblity) = overworld_query.get_single_mut() {
        *overworld_visiblity = Visibility::Visible;
        
        if let Ok(ship_entity) = ship_query.get_single() {
            commands.entity(ship_entity).insert(NewCameraTarget);
        }

        if let Ok(local_island) = island_query.get_single() {
            commands.entity(local_island).remove::<LocalIsland>();
        }
    }
}

fn spawn_overworld(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    overworld_query: Query<&OverworldRoot>,
) {
    if let Ok(_) = overworld_query.get_single() {
        return;
    }

    let overworld_root = commands
        .spawn((
            OverworldRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Visible
        )).id();

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(500))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Ocean
    ))
    .observe(on_clicked_ocean);

    // main island
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(100, 255, 100),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        StarterIsland,
        Island(0),
        Visibility::Inherited
    )).observe(on_clicked_island)
    .set_parent(overworld_root);

    // few smaller islands around the center.
    let island_positions = [
        Vec3::new(10.0, 0.0, 10.0),
        Vec3::new(-10.0, 0.0, 10.0),
        Vec3::new(10.0, 0.0, -10.0),
        Vec3::new(-10.0, 0.0, -10.0),
    ];

    for (i, &pos) in island_positions.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(100, 255, 100),
                ..Default::default()
            })),
            Transform::from_translation(pos),
            Visibility::Inherited,
            Island(i as u64)
        )).observe(on_clicked_island)
        .set_parent(overworld_root);
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
    island_query: Query<(&Transform, &Island, Entity)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut player_enter_event: EventWriter<EnteredIsland>
) {
    if let Ok(ship_transform) = ship_query.get_single() {
        let island_transforms: Vec<(Transform, u64, Entity)> = island_query.iter().map(|(t, i, e)| (*t, i.0, e)).collect();
        
        if let Some((island_id, closest_distance, island_entity)) = island_proximity_check(*ship_transform, island_transforms) {
            let mut proximity_ui_visibility = proximity_ui_query.single_mut();

            if closest_distance < 2.0 {
                *proximity_ui_visibility = Visibility::Inherited;

                if keyboard_input.pressed(KeyCode::KeyF) {
                    state.set(GameState::Island);
                    player_enter_event.send(EnteredIsland(island_id));
                    commands.entity(island_entity).insert(LocalIsland);
                }
            } else {
                *proximity_ui_visibility = Visibility::Hidden;
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
        if let Ok(target) = current_target.get_single() {
            commands.entity(target).remove::<CameraTarget>();
        }
    
        commands.entity(click.entity()).insert(CameraTarget);
    
        println!("Entity {:?} is now the camera target! island", click.entity());
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
        println!("Entity {:?} is now the camera target! ocean", click.entity());

        if let Ok(target) = current_target.get_single() {
            commands.entity(target).remove::<CameraTarget>();
        }

        if let Ok(local_ship) = ship.get_single() {
            commands.entity(local_ship).insert(CameraTarget);
        }
    }
}
