use bevy::prelude::*;

use crate::components::player::{CharacterXp, Gold, Inventory, ItemStack, RewardEvent};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, load_player)
        .add_observer(reward_trigger);
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
    mut xp_query: Query<&mut CharacterXp>,
    mut gold_query: Query<&mut Gold>,
    mut inventory_query: Query<&mut Inventory>
) {
    if let Ok(mut char_xp) = xp_query.single_mut() {
        char_xp.value += trigger.xp;
    } 

    if let Ok(mut player_gold) = gold_query.single_mut() {
        player_gold.value += trigger.gold as u128;
    } 

    if trigger.items.is_some() {
        if let Ok(mut player_inv) = inventory_query.single_mut() {
            for item in trigger.items.as_ref().unwrap().iter() {
                let mut empty_slot: Option<&mut Option<ItemStack>> = None;
                let mut added = false;

                for slot in &mut player_inv.slots {
                    match slot {
                        Some(stack) if stack.id == item.id => {
                            stack.qty += item.qty;
                            added = true;
                            break;
                        }
                        None if empty_slot.is_none() => {
                            empty_slot = Some(slot);
                        }
                        _ => {}
                    }
                }

                if !added {
                    if let Some(slot) = empty_slot {
                        *slot = Some(ItemStack { id: item.id, qty: item.qty });
                    } else {
                        println!("No empty inventory slot for item {}", item.id);
                    }
                }
            }
        } 
    }
}

