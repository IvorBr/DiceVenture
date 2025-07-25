use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct InventoryUIState {
    pub open: bool,
}

#[derive(Component)]
pub struct InventoryPanel;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct XPBar;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct RootUI;

#[derive(Component)]
pub struct GoldText;
