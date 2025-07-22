use std::collections::HashMap;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
}


type ItemId = u64;

#[derive(Clone)]
pub struct ItemStack { pub id: ItemId, pub qty: u16 }

#[derive(Event)]
pub struct RewardEvent {
    pub items: Option<Vec<ItemStack>>,
    pub xp: u64
}

#[derive(Event)]
pub struct SaveEvent;

#[derive(Clone)]
pub struct ItemSpec { pub name: &'static str, pub max: u16, /*icon, etc.*/ }

#[derive(Resource, Default)]
pub struct ItemCatalogue(pub HashMap<ItemId, ItemSpec>);

#[derive(Component)]
pub struct CharacterXp {
    pub value: u64,
    pub level: u64
}

#[derive(Component)]
pub struct Gold {
    pub value: u128,
}