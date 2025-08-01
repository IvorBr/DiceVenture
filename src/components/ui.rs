use bevy::prelude::*;

use crate::plugins::attack::AttackId;

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

#[derive(Component)]
pub struct SkillSlot {
    pub index: usize,
    pub attack_id: AttackId,
}

#[derive(Component)]
pub struct SkillCooldownOverlay;
