use bevy::prelude::*;

use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;

#[derive(Component)]
struct CameraMarker;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, camera_setup)
        .add_systems(Update, camera_update);
    }
}

fn camera_setup(mut commands: Commands
) {
    commands.spawn((
        CameraMarker,
        Camera3dBundle {
            projection: PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                ..default()
            }.into(),
            transform: Transform::from_xyz(0.0, 10.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
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

fn camera_update(
    mut camera_query: Query<&mut Transform, With<CameraMarker>>, 
    players: Query<(&Player, &Transform), (With<Player>, Without<CameraMarker>)>,          
    client: Res<RepliconClient>,                                  
) {
    let client_id = client.id();

    for (player, player_transform) in players.iter() {
        if (client_id.is_some() && player.0 == client_id.unwrap()) || (!client_id.is_some() && player.0 == ClientId::SERVER) {
            if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                camera_transform.translation = Vec3::new(
                    player_transform.translation.x,
                    player_transform.translation.y + 10.0,  
                    player_transform.translation.z + 10.0,
                );

                camera_transform.look_at(
                    player_transform.translation, 
                    Vec3::Y
                );
            }
        }
    }
}