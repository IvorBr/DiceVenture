use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{components::{character::{Character, LocalPlayer}, island::OnIsland}, plugins::camera::PlayerCamera};
use crate::components::overworld::Island;
pub struct DamageNumbersPlugin;
impl Plugin for DamageNumbersPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_observer(spawn_damage_numbers)
        .init_resource::<DamageFont>()
        .add_systems(
            Update,
            animate_numbers,
        );
    }
}

#[derive(Resource, Default)]
struct DamageFont(Handle<Font>);

#[derive(Component)]
struct DamageNumber {
    ttl: Timer,
    rise_speed: f32,
    pos: Vec3
}

#[derive(Event, Serialize, Deserialize)]
pub struct SpawnNumberEvent {
    pub entity: Entity,
    pub amount: u64,
    pub position: IVec3,
}

fn spawn_damage_numbers(
    num_trigger: On<SpawnNumberEvent>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut font: ResMut<DamageFont>,
    local_character: Query<(Entity, &OnIsland), (With<LocalPlayer>, With<Character>)>,
    island_q: Query<&Transform, With<Island>>
) {
    if !font.0.is_strong() {
        font.0 = assets.load("fonts/AncientModernTales.ttf");
    }

    if let Ok((character_entity, island)) = local_character.single() {
        let island_tf = island_q.get(island.0).unwrap();
        let color = if character_entity == num_trigger.entity {
            Color::LinearRgba(LinearRgba { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 })
        } else {
            Color::LinearRgba(LinearRgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 })
        };
        let position = island_tf.translation + num_trigger.position.as_vec3();
        println!("ADDING A DMG NUMBER, pos: {}", position);
        commands.spawn((
            Text::new(num_trigger.amount.to_string()),
            TextFont {
                font: font.0.clone(),
                font_size: 30.0,
                ..default()
            },
            TextColor(color),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            DamageNumber {
                ttl: Timer::from_seconds(0.8, TimerMode::Once),
                rise_speed: 1.5,
                pos: position
            }
        ));
    }
}

fn animate_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Node, &mut TextColor, &mut TextFont, &mut DamageNumber)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
) {
    for (entity, mut node,mut text_color, mut text_font, mut damage_number) in &mut query {
        damage_number.ttl.tick(time.delta());
        if damage_number.ttl.is_finished() {
            commands.entity(entity).despawn();
        }

        if let Ok((camera, cam_transform)) = camera_query.single() {
            damage_number.pos.y += damage_number.rise_speed * time.delta_secs();

            let world_pos = damage_number.pos;
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, world_pos) {
                node.left = Val::Px(screen_pos.x);
                node.top = Val::Px(screen_pos.y);

                let remaining = damage_number.ttl.remaining_secs().max(0.0);
                let delta = remaining / damage_number.ttl.duration().as_secs_f32();
                
                text_font.font_size = 30.0 * (1.0 - 0.5 * (1.0 - delta));

                text_color.0.set_alpha(delta);
            }
        }
    }
}