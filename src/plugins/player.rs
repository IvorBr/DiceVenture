use bevy::prelude::*;

use crate::components::player::{CharacterXp, Gold, Inventory, RewardEvent, SaveEvent};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, load_player)
        .add_observer(reward_trigger)
        .add_observer(save_trigger);
    }
}

fn load_player(
    mut commands: Commands
){
    commands.spawn(CharacterXp{value: 0, level: 0});
    commands.spawn(Gold{value: 0});
    commands.spawn(Inventory::default());
}

fn reward_trigger(
    trigger: Trigger<RewardEvent>,
    commands: Commands,
) {

}

fn save_trigger(
    trigger: Trigger<SaveEvent>,
) {

}

//Do this on client everyone has their own loot...


