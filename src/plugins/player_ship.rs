use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt, ClientTriggerExt};
use crate::components::character::LocalPlayer;
use crate::components::ship::Ship;
use crate::OverworldSet;
use crate::components::overworld::*;
use crate::plugins::camera::{DollyCamera,PlayerCamera};

pub struct PlayerShipPlugin;
impl Plugin for PlayerShipPlugin {
    fn build(&self, app: &mut App) {
        app
        .replicate::<Ship>()
        .add_systems(
            Update,
            (   (
                user_ship_movement,
                ).in_set(OverworldSet),
            )
        );
    }
}

fn user_ship_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera: Query<&DollyCamera, With<PlayerCamera>>,
    mut ship: Query<(Entity, &mut OverworldPosition), (With<Ship>, With<LocalPlayer>)>,
    mut commands: Commands
) {
    if let Ok((entity, mut o_pos)) = ship.single_mut() {
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
        
        if let Ok(camera) = camera.single() {
            direction = match camera.direction {
                0 => direction,
                1 => Vec3::new(direction.z, 0.0, -direction.x),
                2 => Vec3::new(-direction.x, 0.0, -direction.z),
                3 => Vec3::new(-direction.z, 0.0, direction.x),
                _ => direction,
            };
        }

        if direction != Vec3::ZERO {
            let speed = 20.0;
            o_pos.target += direction.normalize() * speed * time.delta_secs();
            
            commands.client_trigger(
                ClientShipPosition{ position: o_pos.target, entity } ,
            );
        }
    }
}