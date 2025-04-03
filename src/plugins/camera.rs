use dolly::prelude::*;
use bevy::prelude::*;
use mint::{Quaternion, Point3};

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
                .with(Arm::new(Point3 {x: 0.0, y: 0.0, z: 15.0}))
                .build(),
            direction: 0
        }
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, camera_setup)
        .add_systems(PreUpdate, change_camera_target)
        .add_systems(Update, (follow_target, rotate_camera, update));
    }
}

fn camera_setup(
    mut commands: Commands
) {
    let transform = Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
    let rotation = transform.rotation;

    commands.spawn((
        PlayerCamera,
        DollyCamera::new(rotation),
        Camera3d {
            ..default()
        },
        transform
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.9),
            illuminance: 2500.0,
            shadows_enabled: true,
            shadow_depth_bias: DirectionalLight::DEFAULT_SHADOW_DEPTH_BIAS,
            shadow_normal_bias: DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS,
        },
        Transform {
            rotation: Quat::from_euler(EulerRot::YXZ, 0.3, -1.0, 0.0),
            ..default()
        },
    ));

    // commands.spawn(DistanceFog {
    //     color: Color::srgb(0.8, 0.85, 1.0),
    //     directional_light_color: Color::srgba(1.0, 0.98, 0.9, 0.3),
    //     directional_light_exponent: 10.0,
    //     falloff: FogFalloff::Exponential { density: 0.01 },
    // });
    
}

fn change_camera_target(
    mut commands: Commands,
    new_targets: Query<Entity, Added<NewCameraTarget>>,
    old_targets: Query<Entity, With<CameraTarget>>,
) {
    if let Ok(entity) = new_targets.get_single() {
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
    if let Ok((maybe_pos, transform)) = target_query.get_single() {
        if let Ok(mut dolly_cam) = camera.get_single_mut() {
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
    if let Ok(mut camera) = camera_query.get_single_mut() {
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

pub fn update(
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