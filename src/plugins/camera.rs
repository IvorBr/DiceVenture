use dolly::prelude::*;
use bevy::prelude::*;
use crate::objects::player::LocalPlayer;

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
pub struct DollyCamera {
    pub rig: CameraRig,
}

impl DollyCamera {
    pub fn new(pos: Vec3, rotation: Quat) -> Self {
        let mut yaw = YawPitch::new();
        yaw.set_rotation_quat(rotation);

        Self {
            rig: CameraRig::builder()
                .with(Position::new(pos))
                .with(yaw)
                .with(Smooth::new_position_rotation(1.0, 1.0))
                .build(),
        }
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, camera_setup)
        .add_systems(Update, (follow_player, update));
    }
}

fn camera_setup(
    mut commands: Commands
) {
    let transform = Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
    let pos = transform.translation;
    let rotation = transform.rotation;

    commands.spawn((
        PlayerCamera,
        DollyCamera::new(pos, rotation),
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
            let offset = Vec3::new(0.0, 10.0, 10.0);

            let target_position = player_transform.translation + offset;
            let pos_driver = dolly_cam.rig.driver_mut::<Position>();
            pos_driver.position = target_position.into();
            let yaw_pitch = dolly_cam.rig.driver_mut::<YawPitch>();
            yaw_pitch.pitch_degrees = -45.0;
            let (_, yaw, _) = player_transform.rotation.to_euler(EulerRot::YXZ);
            yaw_pitch.yaw_degrees = yaw.to_degrees();
        }
    }
}

pub fn update(mut query: Query<(&mut Transform, &mut DollyCamera)>, time: Res<Time>) {
    for (mut transform, mut dolly_cam) in query.iter_mut() {
        dolly_cam.rig.update(time.delta_seconds());
        transform.translation = dolly_cam.rig.final_transform.position.into();
        transform.rotation = dolly_cam.rig.final_transform.rotation.into();
    }
}