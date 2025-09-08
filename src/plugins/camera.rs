use dolly::prelude::*;
use bevy::{asset::RenderAssetUsages, prelude::*};
use mint::{Quaternion, Point3};
use bevy::render::render_resource::*;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct CameraTarget;

#[derive(Component)]
pub struct NewCameraTarget;

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
        .add_systems(Startup, ((create_shader_resources, setup_cameras, setup_light).chain()))
        .add_systems(PreUpdate, change_camera_target)
        .add_systems(Update, (follow_target, rotate_camera, (update_camera, update_render_camera).chain()));
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
        println!("{entity:?} new camera target");
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
    target_query: Query<(Option<&crate::components::humanoid::Position>, &Transform), (With<CameraTarget>, Without<PlayerCamera>)>,
    time: Res<Time>          
) {
    if let Ok((maybe_pos, transform)) = target_query.single() {
        if let Ok(mut dolly_cam) = camera.single_mut() {
            let pos_driver = dolly_cam.rig.driver_mut::<Position>();
            let follow_pos = maybe_pos
                .map(|p| p.0.as_vec3())
                .unwrap_or(transform.translation);
            
            let cur : Vec3 = Vec3::new(pos_driver.position.x, pos_driver.position.y, pos_driver.position.z);
            let new_pos = cur.lerp(follow_pos, time.delta_secs() * 15.0);

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

#[derive(Resource, Clone)]
pub struct CameraColorImage(pub Handle<Image>);

fn create_shader_resources(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width:  1920,
        height: 1080,
        depth_or_array_layers: 1,
    };

    // color texture
    let mut color = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0u8; 8],
        TextureFormat::Rgba16Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    color.texture_descriptor.usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    let color_h = images.add(color);
    commands.insert_resource(CameraColorImage(color_h));
}

use bevy::render::view::RenderLayers;

use crate::plugins::overworld::WATER_HEIGHT;

fn setup_cameras(mut commands: Commands, capture: Res<CameraColorImage>) {
    // Camera 0: world to offscreen images
    commands.spawn((
        Camera {
            order: 0,
            target: bevy::render::camera::RenderTarget::Image(capture.0.clone().into()),
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
        Camera3d {
            ..default()
        },
        RenderLayers::layer(LAYER_WORLD.into()),
        RenderCamera
    ));

    // Camera 1: water and world to screen
    let transform = Transform::from_xyz(0.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
    let rotation = transform.rotation;

    commands.spawn((
        PlayerCamera,
        DollyCamera::new(rotation),
        Camera {
            order : 1,
            ..default()
        },
        Camera3d {
            depth_texture_usages: (TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING).into(),
            ..default()
        },
        RenderLayers::from_layers(&[LAYER_WORLD as usize, LAYER_WATER as usize]),

        transform
    ));
}

pub fn update_render_camera(
    main_query: Query<&Transform, (With<PlayerCamera>, Without<RenderCamera>)>,
    mut capture_query: Query<&mut Transform, (With<RenderCamera>, Without<PlayerCamera>)>,
) {
    if let (Ok(main_transform), Ok(mut capture_transform)) = (main_query.single(), capture_query.single_mut()) {
        let mut projection = main_transform.translation;
        projection.y = 2.0 * WATER_HEIGHT - projection.y;

        let forward = main_transform.forward();
        let up = main_transform.up();
        let forward_mirrored = Vec3::new(forward.x, -forward.y, forward.z).normalize();
        let mut up_mirrored = Vec3::new(up.x, -up.y, up.z).normalize();

        let right_mirrored = forward_mirrored.cross(up_mirrored).normalize();
        up_mirrored = right_mirrored.cross(forward_mirrored).normalize();

        let rot = Quat::from_mat3(&Mat3::from_cols(right_mirrored, up_mirrored, -forward_mirrored));

        *capture_transform = Transform {
            translation: projection,
            rotation: rot,
            scale: Vec3::new(1.0,1.0,1.0),
        };
    }
}