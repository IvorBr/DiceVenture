use bevy::prelude::*;
use crate::GameState;
use crate::objects::overworld::*;
use crate::plugins::camera::CameraTarget;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};


#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(0)]
    pub wave_strength: f32,
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::rgb(0.0,0.505,0.505),
            wave_strength: 0.1,
        }
    }
}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders\\water_shader.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders\\water_shader.wgsl".into()
    }
}

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((MeshPickingPlugin, MaterialPlugin::<WaterMaterial>::default()))
        .add_systems(OnEnter(GameState::Overworld), (spawn_overworld, spawn_ship))
        .add_systems(
            Update,
                ship_movement_system.run_if(in_state(GameState::Overworld))
        );
    }
}

fn spawn_overworld(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            ..Default::default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
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
    )).observe(on_clicked_island);

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
        )).observe(on_clicked_island);
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

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.65, 0.45, 0.25),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, -0.25, 0.75),
        Ship,
        CameraTarget
    ));
}

fn get_wave_height(position: Vec3, time: f32, wave_strength: f32) -> f32 {
    (position.x + time).sin() * (position.z + time).sin() * wave_strength
}

fn get_wave_normal(position: Vec3, time: f32, wave_strength: f32) -> Vec3 {
    let epsilon = 0.1;
    
    let dx = Vec3::new(epsilon, 0.0, 0.0);
    let dz = Vec3::new(0.0, 0.0, epsilon);

    let height_center = get_wave_height(position, time, wave_strength);
    let height_x = get_wave_height(position + dx, time, wave_strength);
    let height_z = get_wave_height(position + dz, time, wave_strength);

    let normal = Vec3::normalize(Vec3::new(
        height_x - height_center,
        epsilon,
        height_z - height_center
    ));

    normal
}

fn ship_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Ship>>,
    water_materials: Res<Assets<WaterMaterial>>,
) {
    let wave_strength = water_materials
        .iter()
        .next()
        .map(|(_, mat)| mat.wave_strength)
        .unwrap_or(0.1);

    for mut transform in &mut query {
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
            transform.translation += direction.normalize() * speed * time.delta_secs();
        }

        let wave_height = get_wave_height(transform.translation, time.elapsed_secs(), wave_strength);
        transform.translation.y = wave_height;

        let wave_normal = get_wave_normal(transform.translation, time.elapsed_secs(), wave_strength);
        let rotation = Quat::from_rotation_arc(Vec3::Y, wave_normal);
        transform.rotation = rotation * Quat::from_rotation_y(transform.rotation.to_euler(EulerRot::XYZ).0);

    }
}
