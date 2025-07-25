use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::attacks::base_attack::BaseAttackPlugin;
use crate::attacks::cut_through::CutThroughPlugin;
use crate::components::enemy::Enemy;
use crate::components::humanoid::{AttackCooldowns, Health};
use crate::components::island_maps::IslandMaps;
use crate::components::character::LocalPlayer;
use crate::components::player::RewardEvent;
use crate::plugins::damage_numbers::SpawnNumberEvent;
use crate::preludes::network_preludes::*;
use std::collections::HashMap;

use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use twox_hash::XxHash64;

pub type AttackId = u64;

/// Unique key per attack
pub fn key_of<T: 'static>() -> AttackId {
    let mut hash = BuildHasherDefault::<XxHash64>::default().build_hasher();
    hash.write(std::any::type_name::<T>().as_bytes());
    hash.finish()
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct ClientAttack {
   pub attack_id: AttackId,
    pub offset: IVec3
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct AttackInfo{
    pub attack_id: AttackId,
    pub offset: IVec3
}

pub type SpawnFunction = fn(&mut Commands, Entity, IVec3);

#[derive(Resource, Default)]
pub struct AttackRegistry {
    map: HashMap<AttackId, SpawnFunction>,
}
impl AttackRegistry {
    pub fn register<T: 'static>(&mut self, func: SpawnFunction) -> AttackId {
        let key = key_of::<T>();
        self.map.insert(key, func);
        key
    }
    
    pub fn spawn(&self, key: AttackId, commands: &mut Commands, entity: Entity, offset: IVec3) {
        if let Some(func) = self.map.get(&key) { 
            func(commands, entity, offset) 
        }
        else { error!("Unknown attack key {key}") }
    }
}

#[derive(Clone)]
pub struct AttackSpec {
    pub offsets : &'static[IVec3],
    pub cooldown : f32,
    pub damage: u64
}

#[derive(Resource, Default)]
pub struct AttackCatalogue(pub HashMap<AttackId, AttackSpec>);

pub struct AttackPlugin;
impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(AttackRegistry::default())
        .insert_resource(AttackCatalogue::default())
        .add_client_trigger::<ClientAttack>(Channel::Unordered)
        .add_server_trigger::<AttackInfo>(Channel::Unordered)
        .add_server_trigger::<ClientDamageEvent>(Channel::Unordered)
        .add_observer(server_apply_attack)
        .add_observer(client_visualize_attack)
        .add_observer(client_damage_trigger)
        .add_observer(damage_trigger)
        .add_observer(attack_trigger)
        .add_systems(PreUpdate, tick_attack_cooldowns.run_if(server_running))
        .add_plugins((BaseAttackPlugin, CutThroughPlugin));
    }
}

fn server_apply_attack(
    client_trigger: Trigger<FromClient<ClientAttack>>,
    mut commands: Commands,
) {
    let msg = client_trigger.event();
    let attacker = client_trigger.target();

    commands.server_trigger_targets(
        ToClients {
            mode : SendMode::BroadcastExcept(client_trigger.client_entity),
            event: AttackInfo { attack_id: msg.attack_id, offset: msg.offset },
        },
        attacker,
    );
}

fn client_visualize_attack(
    server_trigger: Trigger<AttackInfo>,
    mut commands: Commands,
    attack_reg: Res<AttackRegistry>
) {
    attack_reg.spawn(server_trigger.attack_id, &mut commands, server_trigger.target(), server_trigger.offset);
}

fn tick_attack_cooldowns(
    mut cooldowns: Query<&mut AttackCooldowns>,
    time: Res<Time>,
) {
    for mut cooldown in &mut cooldowns {
        for timer in cooldown.0.values_mut() {
            timer.tick(time.delta());
        }
    }
}

#[derive(Event)]
pub struct DamageEvent {
    island: u64,
    offset: IVec3,
    damage: u64
}

impl DamageEvent {
    pub fn new(island: u64, offset: IVec3, damage: u64) -> Self {
        Self { island, offset, damage }
    }
}

#[derive(Event, Serialize, Deserialize)]
pub struct ClientDamageEvent {
    amount: u64,
    position: IVec3,
    remaining_health: u64
}

fn client_damage_trigger(
    damage_trigger: Trigger<ClientDamageEvent>,
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>
){
    commands.trigger(SpawnNumberEvent {amount: damage_trigger.amount, position: damage_trigger.position, entity: damage_trigger.target()} );

    if damage_trigger.remaining_health == 0 {
        if let Ok(enemy_entity) = enemies.get(damage_trigger.target()) {
            // TODO: get the type of the enemy and its possible loot, xp and gold
            commands.trigger(RewardEvent {
                items: None,
                xp: 1,
                gold: 0,
            });
        }
    }   
}

fn damage_trigger(
    damage_trigger: Trigger<DamageEvent>,
    island_maps: Res<IslandMaps>,
    mut health: Query<&mut Health>,
    server: Option<Res<RenetServer>>,
    mut commands: Commands
) {
    if server.is_some() {
        if let Some(map) = island_maps.maps.get(&damage_trigger.island) {
            if let Some(victim) = map.get_target(damage_trigger.offset) {
                if let Ok(mut hp) = health.get_mut(victim) {
                    let remaining_health = hp.damage(damage_trigger.damage);

                    commands.server_trigger_targets(
                        ToClients {
                            mode : SendMode::Broadcast,
                            event: ClientDamageEvent {amount: damage_trigger.damage, position: damage_trigger.offset, remaining_health: remaining_health},
                        },
                        victim
                    );
                }
            }
        }
    }
}

#[derive(Event)]
pub struct AttackEvent {
    entity: Entity,
    attack_id: AttackId,
    offset: IVec3,
}

impl AttackEvent {
    pub fn new( entity: Entity, attack_id: AttackId, offset: IVec3) -> Self {
        Self { entity, attack_id, offset }
    }
}

fn attack_trigger(
    attack_trigger: Trigger<AttackEvent>,
    mut commands: Commands,
    attack_reg: Res<AttackRegistry>,
    attack_cat: Res<AttackCatalogue>,
    mut cooldowns_query: Query<&mut AttackCooldowns, With<LocalPlayer>>
) {
    if let Ok(cooldowns) = &mut cooldowns_query.single_mut() {
        if let Some(timer) = cooldowns.0.get_mut(&attack_trigger.attack_id) {
            if !timer.finished() { 
                return; 
            }
        }

        cooldowns.0.insert(attack_trigger.attack_id, Timer::from_seconds(attack_cat.0.get(&attack_trigger.attack_id).unwrap().cooldown, TimerMode::Once));
    }

    commands.client_trigger_targets(
        ClientAttack {
            attack_id: attack_trigger.attack_id,
            offset: attack_trigger.offset
        },
        attack_trigger.entity
    );

    attack_reg.spawn(attack_trigger.attack_id, &mut commands, attack_trigger.entity, attack_trigger.offset);
}