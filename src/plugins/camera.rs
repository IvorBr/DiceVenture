use bevy::camera::visibility::RenderLayers;
use dolly::prelude::*;
use bevy::{core_pipeline::prepass::DepthPrepass, prelude::*};
use mint::{Quaternion, Point3};
use bevy::render::render_resource::*;
use crate::components::overworld::Ocean;
use crate::plugins::overworld::WATER_HEIGHT;
use crate::GameState;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct CameraTarget;

#[derive(Component)]
pub struct NewCameraTarget;

#[derive(Component)]
pub struct ZoomCamera(pub f32);

#[derive(Component)]
pub struct DollyCamera {
    pub rig: CameraRig,
    pub direction: u8
}

#[derive(Component)]
pub struct RenderCamera;

pub const LAYER_WORLD: u8 = 0;
pub const LAYER_WATER: u8 = 1;

impl DollyCamera {
    pub fn new(rotation: Quat) -> Self {
        let mut yaw = YawPitch::new();

        yaw.set_rotation_quat(Quaternion {
            s: rotation.w,
            v: mint::Vector3 {
                x: rotation.x,
                y: rotation.y,
                z: rotation.z,
            },
        });

        Self {
            rig: CameraRig::builder()
                .with(Position::new(Point3 {x: 0.0, y: 0.0, z: 0.0}))
                .with(yaw)
                .with(Smooth::new_rotation(1.0))
                .with(Arm::new(Point3 {x: 0.0, y: 0.0, z: 18.0}))
                .build(),
            direction: 0
        }
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::Island), zoom_in_camera)
        .add_systems(OnEnter(GameState::Overworld), zoom_out_camera)
        .add_systems(Startup, (setup_light, setup_cameras))
        .add_systems(PreUpdate, (resize_ocean_to_camera, zoom_to_distance, change_camera_target, follow_target, rotate_camera, update_camera));
    }
}

fn resize_ocean_to_camera(
    mut ocean_q: Query<&mut Transform, With<Ocean>>,
    cam_q: Query<(&GlobalTransform, &DollyCamera), With<PlayerCamera>>,
) {
    let Ok((cam_gt, dolly)) = cam_q.single() else { return; };
    let mut t = match ocean_q.single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    let cam_pos = cam_gt.translation();
    t.translation.x = cam_pos.x;
    t.translation.z = cam_pos.z;

    let arm = dolly.rig.driver::<Arm>();
    let cur = arm.offset;
    let dist = (cur.x * cur.x + cur.y * cur.y + cur.z * cur.z).sqrt();

    let target_size = (dist * 6.0).clamp(75.0, 6000.0);

    let s = target_size / 75.0;
    t.scale = Vec3::splat(s);
    t.translation.y = WATER_HEIGHT;
}

pub fn zoom_to_distance(
    mut q: Query<(&mut DollyCamera, &ZoomCamera)>, 
    time: Res<Time>, 
) {
    let speed = 5.0;
    for (mut dolly, zoom) in &mut q {
        let arm = dolly.rig.driver_mut::<Arm>();
        let cur = arm.offset;
        let cur_len = (cur.x * cur.x + cur.y * cur.y + cur.z * cur.z).sqrt();
        if cur_len > 0.0 {
            let t = 1.0 - f32::exp(-speed * time.delta_secs());
            let new_len = cur_len + (zoom.0 - cur_len) * t;
            let dir = mint::Vector3 {
                x: cur.x / cur_len,
                y: cur.y / cur_len,
                z: cur.z / cur_len,
            };
            arm.offset = Point3 {
                x: dir.x * new_len,
                y: dir.y * new_len,
                z: dir.z * new_len,
            }.into();
        }
    }
}

pub fn zoom_in_camera(
    mut commands: Commands,
    query: Query<Entity, With<DollyCamera>>, 
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ZoomCamera(16.0));
        //set camera state to zoomed in
    }
}

pub fn zoom_out_camera(
    mut commands: Commands,
    query: Query<Entity, With<DollyCamera>>, 
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ZoomCamera(18.0*6.0));
        //set camera state to zoomed out
    }
}

fn setup_light(
    mut commands: Commands,
) {
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 1.0, 1.0),
            illuminance: 2500.0,
            shadows_enabled: true,
            affects_lightmapped_mesh_diffuse: true,
            shadow_depth_bias: DirectionalLight::DEFAULT_SHADOW_DEPTH_BIAS,
            shadow_normal_bias: DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS,
        },
        Transform {
            rotation: Quat::from_euler(EulerRot::YXZ, 0.3, -1.0, 0.0),
            ..default()
        },
    ));
}

fn change_camera_target(
    mut commands: Commands,
    new_targets: Query<Entity, Added<NewCameraTarget>>,
    old_targets: Query<Entity, With<CameraTarget>>,
) {
    if let Ok(entity) = new_targets.single() {
        commands.entity(entity)
            .remove::<NewCameraTarget>()
            .insert(CameraTarget);

        for entity in old_targets.iter() {
            commands.entity(entity).remove::<CameraTarget>();
        }
    }
}

fn follow_target(
    mut camera: Query<&mut DollyCamera, With<PlayerCamera>>, 
    target_query: Query<&GlobalTransform, (With<CameraTarget>, Without<PlayerCamera>)>,
    time: Res<Time>          
) {
    if let Ok(target_transform) = target_query.single() {
        if let Ok(mut dolly_cam) = camera.single_mut() {
            let pos_driver = dolly_cam.rig.driver_mut::<Position>();
            let follow_pos = target_transform.translation();
            
            let current: Vec3 = Vec3::new(pos_driver.position.x, pos_driver.position.y, pos_driver.position.z);
            let new_pos = current.lerp(follow_pos, time.delta_secs() * 15.0);

            pos_driver.position = new_pos.to_array().into();
        }
    }
}

pub fn rotate_camera(
    mut camera_query: Query<&mut DollyCamera>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut camera) = camera_query.single_mut() {
        if input.just_pressed(KeyCode::KeyQ) {
            camera.rig.driver_mut::<YawPitch>().rotate_yaw_pitch(90.0, 0.0);
            camera.direction = (camera.direction + 1) % 4;
        }
        if input.just_pressed(KeyCode::KeyE) {
            camera.rig.driver_mut::<YawPitch>().rotate_yaw_pitch(-90.0, 0.0);
            camera.direction = (camera.direction + 3) % 4;
        }
    }
}

pub fn update_camera(
    mut query: Query<(&mut Transform, &mut DollyCamera)>, 
    time: Res<Time>
) {
    for (mut transform, mut dolly_cam) in query.iter_mut() {
        dolly_cam.rig.update(time.delta_secs());
        let dolly_pos = dolly_cam.rig.final_transform.position;
        let dolly_rot = dolly_cam.rig.final_transform.rotation;
        transform.translation = Vec3::new(dolly_pos.x, dolly_pos.y, dolly_pos.z);
        transform.rotation = Quat::from_xyzw(dolly_rot.v.x, dolly_rot.v.y, dolly_rot.v.z, dolly_rot.s);
    }
}


fn setup_cameras(mut commands: Commands) {
    let transform = Transform::from_translation(Vec3::new(0.0, 12.0, 10.0)).looking_at(Vec3::ZERO, Vec3::Y);
    let rotation = transform.rotation;

    commands.spawn((
        PlayerCamera,
        DollyCamera::new(rotation),
        Camera3d {
            depth_texture_usages: (TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING).into(),
            ..default()
        },
        RenderLayers::from_layers(&[LAYER_WORLD as usize, LAYER_WATER as usize]),
        DepthPrepass,
        transform
    ));
}