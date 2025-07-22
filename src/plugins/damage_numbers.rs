use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientTriggerAppExt, ServerTriggerAppExt};
use serde::{Deserialize, Serialize};

pub struct DamageNumbersPlugin;
impl Plugin for DamageNumbersPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_server_trigger::<SpawnNumberEvent>(Channel::Unordered)
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
}

#[derive(Event, Serialize, Deserialize)]
pub struct SpawnNumberEvent {
    pub amount: u64,
    pub position: IVec3,
}


fn spawn_damage_numbers(
    num_trigger: Trigger<SpawnNumberEvent>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut font: ResMut<DamageFont>,
) {
    println!("REACHED");
    if font.0.is_weak() {
        font.0 = assets.load("fonts/AncientModernTales.ttf");
    }

    
    commands.spawn((
        Text::new(num_trigger.amount.to_string()),
        TextFont {
            font: font.0.clone(),
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation((Vec3{x: num_trigger.position.x as f32, y: (num_trigger.position.y + 1) as f32, z: num_trigger.position.z as f32}) * 1.5),
        DamageNumber {
            ttl: Timer::from_seconds(0.8, TimerMode::Once),
            rise_speed: 1.5,
        },
    ));

    //     commands.spawn((
    //     Text2d::new("scale"),
    //     text_font,
    //     TextLayout::new_with_justify(text_justification),
    //     Transform::from_translation(Vec3::new(400.0, 0.0, 0.0)),
    //     AnimateScale,
    // ));
}

fn animate_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageNumber)>,
) {
    for (entity, mut transform, mut text_color, mut damage_number) in &mut query {
        damage_number.ttl.tick(time.delta());
        //transform.translation.y += damage_number.rise_speed * time.delta_secs();

        let remaining = damage_number.ttl.remaining_secs().max(0.0);
        let alpha = remaining / damage_number.ttl.duration().as_secs_f32();
        text_color.0.set_alpha(alpha);

        if damage_number.ttl.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
