use bevy::{math::VectorSpace, prelude::*, state::commands};
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
        .add_systems(OnEnter(GameState::Overworld), (spawn_overworld, spawn_overworld_ui))
        .add_systems(OnExit(GameState::Overworld), overworld_cleanup)
        .add_systems(
            Update,
            (
                ship_movement_system.run_if(in_state(GameState::Overworld)),
                island_proximity.run_if(in_state(GameState::Overworld)),
            )
        );
    }
}

fn overworld_cleanup(
    mut commands: Commands,
    ui_query: Query<Entity, With<OverworldUI>>,
    overworld_query: Query<Entity, With<OverworldRoot>>,

) {
    if let Ok(ui_entity) = ui_query.get_single() {
        commands.entity(ui_entity).despawn_recursive();
    }

    if let Ok(overworld_entity) = overworld_query.get_single() {
        commands.entity(overworld_entity).insert(Visibility::Hidden);
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

fn spawn_overworld(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    overworld_query: Query<&OverworldRoot>
) {
    if let Ok(_) = overworld_query.get_single() {
        return;
    }

    let overworld_root = commands
        .spawn((
            OverworldRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            InheritedVisibility::VISIBLE
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
        StarterIsland
    )).observe(on_clicked_island)
    .set_parent(overworld_root);

    // few smaller islands around the center.
    let island_positions = [
        Vec3::new(10.0, 0.0, 10.0),
        Vec3::new(-10.0, 0.0, 10.0),
        Vec3::new(10.0, 0.0, -10.0),
        Vec3::new(-10.0, 0.0, -10.0),
    ];

    for pos in island_positions {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(100, 255, 100),
                ..Default::default()
            })),
            Transform::from_translation(pos),
            Island
        )).observe(on_clicked_island)
        .set_parent(overworld_root);
    }

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.65, 0.45, 0.25),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.75),
        Ship,
        NewCameraTarget
    )).set_parent(overworld_root);
}

fn island_proximity_check(
    position: Transform,
    islands: Vec<(Entity, Transform)> 
) -> Option<(Entity, f32)> {
    if islands.is_empty() {
        return None;
    }

    let (mut best_island, island_transform) = islands[0];
    let mut best_distance = position.translation.distance_squared(island_transform.translation);

    for (entity, transform) in islands.iter().skip(1) {
        let distance = position.translation.distance_squared(transform.translation);
        if distance < best_distance {
            best_distance = distance;
            best_island = *entity;
        }
    }

    Some((best_island, best_distance))
}

fn island_proximity(
    mut commands: Commands,
    mut proximity_ui_query: Query<&mut Visibility, With<ProximityUI>>,
    ship_query: Query<&Transform, With<Ship>>,
    island_query: Query<(Entity, &Transform), With<Island>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
) {
    if let Ok(ship_transform) = ship_query.get_single() {
        let island_transforms: Vec<(Entity, Transform)> = island_query.iter().map(|(e, t)| (e, *t)).collect();
        
        if let Some((entity, closest_distance)) = island_proximity_check(*ship_transform, island_transforms) {
            let mut proximity_ui_visibility = proximity_ui_query.single_mut();

            if closest_distance < 2.0 {
                *proximity_ui_visibility = Visibility::Inherited;

                if keyboard_input.pressed(KeyCode::KeyF) {
                    commands.entity(entity).insert(SelectedIsland);
                    state.set(GameState::Island);
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
    current_target: Query<Entity, With<CameraTarget>>
) {
    if let Ok(target) = current_target.get_single() {
        commands.entity(target).remove::<CameraTarget>();
    }

    commands.entity(click.entity()).insert(CameraTarget);

    println!("Entity {:?} is now the camera target!", click.entity());
}

fn on_clicked_ocean(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    current_target: Query<Entity, (With<CameraTarget>,Without<Ship>)>,
    ship: Query<Entity, (With<Ship>, Without<CameraTarget>)>
) {
    println!("Entity {:?} is now the camera target!", click.entity());

    if let Ok(target) = current_target.get_single() {
        commands.entity(target).remove::<CameraTarget>();
    }

    if let Ok(local_ship) = ship.get_single() {
        commands.entity(local_ship).insert(CameraTarget);
    }
}

fn ship_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ship: Query<&mut Transform, (With<Ship>, Without<Ocean>)>,
    mut ocean: Query<&mut Transform, (Without<Ship>, With<Ocean>)>,
) {
    for mut ship_transform in &mut ship {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec3::ZERO {
            let speed = 5.0;
            ship_transform.translation += direction.normalize() * speed * time.delta_secs();
            
            for mut ocean_transform in &mut ocean {
                ocean_transform.translation.x = ship_transform.translation.x;
                ocean_transform.translation.z = ship_transform.translation.z;
            }
        }
    }
}
