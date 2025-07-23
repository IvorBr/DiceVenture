use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ServerTriggerAppExt};
use serde::{Deserialize, Serialize};
use crate::{components::character::{Character, LocalPlayer}, plugins::camera::PlayerCamera};

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
    num_trigger: Trigger<SpawnNumberEvent>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut font: ResMut<DamageFont>,
    local_character: Query<Entity, (With<LocalPlayer>, With<Character>)>
) {
    if font.0.is_weak() {
        font.0 = assets.load("fonts/AncientModernTales.ttf");
    }

    if let Ok(character_entity) = local_character.single() {
        let color = if character_entity == num_trigger.entity {
            Color::LinearRgba(LinearRgba { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 })
        } else {
            Color::LinearRgba(LinearRgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 })
        };

        commands.spawn((
            Text::new(num_trigger.amount.to_string()),
            TextFont {
                font: font.0.clone(),
                font_size: 30.0,
                ..default()
            },
            TextColor(color),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            DamageNumber {
                ttl: Timer::from_seconds(0.8, TimerMode::Once),
                rise_speed: 1.5,
                pos: Vec3::new(num_trigger.position.x as f32, (num_trigger.position.y + 1) as f32, num_trigger.position.z as f32)
            },
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
        if damage_number.ttl.finished() {
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

// fn spawn_damage_numbers(
//     num_trigger: Trigger<SpawnNumberEvent>,
//     mut commands: Commands,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     commands.spawn((
//         Text3d::new(num_trigger.amount.to_string()),
//         Text3dStyling {
//             size: 1.0,
//             color: Color::WHITE.into(),
//             //font: "Ancient Modern Tales".into(),
//             ..Default::default()
//         },
//         Mesh3d::default(),
//         MeshMaterial3d(materials.add(
//         StandardMaterial {
//             base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
//             alpha_mode: AlphaMode::Blend,
//             ..Default::default()
//         })),
//         Transform::from_translation(Vec3::new(num_trigger.position.x as f32, (num_trigger.position.y + 5) as f32, num_trigger.position.z as f32)),
//         DamageNumber {
//             ttl: Timer::from_seconds(0.8, TimerMode::Once),
//             rise_speed: 1.5,
//         },
//     ));
// }

// fn animate_numbers(
//     mut commands: Commands,
//     time: Res<Time>,
//     mut query: Query<(Entity, &mut Transform, &mut Text3dStyling, &mut DamageNumber)>,
// ) {
//     for (entity, mut transform, mut text_styling, mut damage_number) in &mut query {
//         damage_number.ttl.tick(time.delta());
//         transform.translation.y += damage_number.rise_speed * time.delta_secs();

//         let remaining = damage_number.ttl.remaining_secs().max(0.0);
//         let alpha = remaining / damage_number.ttl.duration().as_secs_f32();
//         text_styling.color.alpha = alpha;

//         if damage_number.ttl.finished() {
//             commands.entity(entity).despawn();
//         }
//     }
// }
