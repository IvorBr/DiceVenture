use dolly::prelude::*;
use bevy::prelude::*;
use crate::objects::player::LocalPlayer;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct DollyCamera {
    pub rig: CameraRig,
    pub direction: u8
}

impl DollyCamera {
    pub fn new(rotation: Quat) -> Self {
        let mut yaw = YawPitch::new();
        yaw.set_rotation_quat(rotation);

        Self {
            rig: CameraRig::builder()
                .with(Position::new(Vec3::ZERO))
                .with(yaw)
                .with(Smooth::new_rotation(1.0))
                .with(Arm::new(Vec3::Z * 15.0))
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
        .add_systems(Update, (follow_player, rotate_camera, update));
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
        Camera3dBundle {
            transform,
            ..default()
        },
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
   
fn follow_player(
    mut camera: Query<&mut DollyCamera, With<PlayerCamera>>, 
    player: Query<&Transform, (With<LocalPlayer>, Without<PlayerCamera>)>,          
) {
    if let Ok(player_transform) = player.get_single() {
        if let Ok(mut dolly_cam) = camera.get_single_mut() {
            let pos_driver = dolly_cam.rig.driver_mut::<Position>();
            pos_driver.position = player_transform.translation.into();
        }
    }
}

pub fn rotate_camera(
    mut camera_query: Query<&mut DollyCamera>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut camera) = camera_query.get_single_mut() {
        if input.just_pressed(KeyCode::ArrowRight) {
            camera.rig.driver_mut::<YawPitch>().rotate_yaw_pitch(90.0, 0.0);
            camera.direction = (camera.direction + 1) % 4;
        }
        if input.just_pressed(KeyCode::ArrowLeft) {
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
        dolly_cam.rig.update(time.delta_seconds());
        transform.translation = dolly_cam.rig.final_transform.position.into();
        transform.rotation = dolly_cam.rig.final_transform.rotation.into();
    }
}