use bevy::prelude::*;
use crate::components::{character::LocalPlayer, humanoid::Health, player::{CharacterXp, Gold, Inventory}, ui::*};

const BORDER_RADIUS : Val = Val::Px(5.0);
const XP_BAR_WIDTH : f32 = 100.0;
const BASE_FONT_SIZE : f32 = 18.0;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(InventoryUIState::default())
        .add_systems(Startup, setup_ui)
        .add_systems(Update, (inventory_controls, xp_changed, character_health_changed, gold_changed, inventory_update));
    }
}

fn setup_ui(
    mut commands: Commands, 
) {
    commands.spawn((
        // Full screen root UI, can be used to easily hide UI
        RootUI,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexEnd, 
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Percent(1.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
    .with_children(|parent| {
        // HP background
        parent.spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BorderRadius::all(BORDER_RADIUS),
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.0)),
        )).with_children(|hp_box| {
            //HP text
            hp_box.spawn((
                Text::new("HP"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: BASE_FONT_SIZE,
                    ..default()
                },
                HealthText,
            ));
        });

        // XP background
        parent.spawn((
            Node {
                width: Val::Px(XP_BAR_WIDTH),
                height: Val::Px(20.0),
                position_type: PositionType::Relative,
                ..default()
            },
            BorderRadius::all(BORDER_RADIUS),
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.0)),
        ))
        .with_children(|bar| {
            // XP progress bar
            bar.spawn((
                Node {
                    width: Val::Px(XP_BAR_WIDTH),
                    height: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.875, 0.0, 1.0)),
                BorderRadius::all(BORDER_RADIUS),
                XPBar,
            ));

            // Level number
            bar.spawn((
                Text::new("1"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: BASE_FONT_SIZE,
                    ..default()
                },
                LevelText,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    margin: UiRect {
                        left: Val::Px(-BASE_FONT_SIZE/2.0),
                        top: Val::Px(-BASE_FONT_SIZE/2.0),
                        ..default()
                    },
                    ..default()
                },
            ));
        });

        parent.spawn((
            //Gold Background
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(1.0),
                right: Val::Percent(1.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect::all(Val::Px(4.0)),                
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.0)),
            BorderRadius::all(BORDER_RADIUS),
        ))
        .with_children(|gold_box| {
            //Gold text
            gold_box.spawn((
                Text::new("GOLD"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: BASE_FONT_SIZE,
                    ..default()
                },
                GoldText,
            ));
        });
    });

    // Inventory UI
    commands.spawn((
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            width: Val::Px(300.0),
            height: Val::Px(300.0),
            flex_wrap: FlexWrap::Wrap,
            padding: UiRect::all(Val::Px(4.0)),
            ..Default::default()
        },
        BorderRadius::all(BORDER_RADIUS),
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.0)),
        InventoryPanel,
    ));
}

fn inventory_controls(
    input: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<InventoryUIState>,
    mut inventory_query: Query<&mut Node, With<InventoryPanel>>,
) {
    if input.just_pressed(KeyCode::KeyI) {
        ui_state.open = !ui_state.open;
        for mut node in &mut inventory_query {
            node.display = if ui_state.open {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

fn inventory_update(
    mut ui_query: Query<(Entity, &mut Children), With<InventoryPanel>>,
    mut commands: Commands,
    inventory_query: Query<&Inventory, Changed<Inventory>>,
) {
    let Ok(inventory) = inventory_query.single() else { return };
    let Ok((panel_entity, children)) = ui_query.single_mut() else { return };

    for child in children.iter() {
        commands.entity(child).despawn();
    }

    for slot in inventory.slots.iter() {
        if let Some(stack) = slot {
            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn((
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(BORDER_RADIUS),
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                ))
                .with_children(|item| {
                    item.spawn((
                        Text::new(format!("{} ×{}", stack.id, stack.qty)),
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                    ));
                });
            });
        }
    }
}

fn xp_changed(
    mut xp_ui_query: Query<&mut Node, With<XPBar>>,
    xp_query: Query<&CharacterXp, Changed<CharacterXp>>,
) {
    let Ok(xp) = xp_query.single() else { return };
    let Ok(mut node) = xp_ui_query.single_mut() else { return };

    let progress = xp.value as f32 / 5 as f32;
    node.width = Val::Px(XP_BAR_WIDTH * progress.clamp(0.0, 1.0));
}

fn gold_changed(
    mut ui_query: Query<&mut Text, With<GoldText>>,
    gold_query: Query<&Gold, Changed<Gold>>,
) {
    let Ok(gold) = gold_query.single() else { return };
    let Ok(mut text) = ui_query.single_mut() else { return };

    text.0 = gold.value.to_string() + " Gold";
}

fn character_health_changed(
    mut health_ui_query: Query<&mut Text, With<HealthText>>,
    health_query: Query<&Health, (Changed<Health>, With<LocalPlayer>)>,
) {
    let Ok(health) = health_query.single() else { return };
    let Ok(mut text) = health_ui_query.single_mut() else { return };

    text.0 = health.value.to_string() + " HP";
}

