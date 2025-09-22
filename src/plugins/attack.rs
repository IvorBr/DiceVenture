use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::attacks::base_attack::BaseAttackPlugin;
use crate::attacks::counter::CounterPlugin;
use crate::attacks::cut_through::CutThroughPlugin;
use crate::attacks::dagger_throw::DaggerThrowPlugin;
use crate::components::enemy::Enemy;
use crate::components::humanoid::{ActiveSkills, AttackCooldowns, DamageVisualizer, Health, Stunned, ViewDirection, VisualEntity, VisualRef};
use crate::components::island::OnIsland;
use crate::components::island_maps::IslandMaps;
use crate::components::player::RewardEvent;
use crate::plugins::damage_numbers::SpawnNumberEvent;
use crate::plugins::projectiles::ProjectilePlugin;
use crate::preludes::network_preludes::*;
use crate::CHUNK_SIZE;
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

#[derive(Component)]
pub struct AttackMarker;

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
        .add_mapped_server_trigger::<NegateDamageTrigger>(Channel::Unordered)
        .add_event::<NegatedDamageEvent>()
        .add_observer(server_apply_attack)
        .add_observer(client_visualize_attack)
        .add_observer(client_damage_trigger)
        .add_observer(damage_trigger)
        .add_observer(attack_trigger)
        .add_observer(damage_negated_trigger)
        .add_systems(PreUpdate, (tick_attack_cooldowns, interrupt_attack_stun, damage_visualizer_system))
        .add_plugins((BaseAttackPlugin, CutThroughPlugin, DaggerThrowPlugin, CounterPlugin, ProjectilePlugin));
    }
}

#[derive(Event, Deserialize, Serialize, MapEntities)]
pub struct NegateDamageTrigger {
    pub attack_id: u64,
    #[entities]
    pub owner: Entity,
    #[entities]
    pub victim: Entity,
    pub island: u64,
    pub offset: IVec3,
    pub damage: u64,
}

fn damage_negated_trigger(
    trigger: Trigger<NegateDamageTrigger>,
    mut process_writer: EventWriter<NegatedDamageEvent>,
    active_skills_q: Query<&ActiveSkills>
){
    if let Ok(active_skills) = active_skills_q.get(trigger.victim){
        if let Some(skill_entity) = active_skills.0.get(&trigger.attack_id) {
            process_writer.write(NegatedDamageEvent {
                owner: trigger.owner,
                victim: *skill_entity,
                island: trigger.island,
                offset: trigger.offset,
                damage: trigger.damage,
            });
        }
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
    attack_reg: Res<AttackRegistry>,
    mut active_skills_q: Query<&mut ActiveSkills>,
    mut view_direction_q: Query<&mut ViewDirection>
) {
    if let Ok(mut view_direction) = view_direction_q.get_mut(server_trigger.target()) {
        view_direction.0 = server_trigger.offset;
    }

    if let Ok(mut active_skills) = active_skills_q.get_mut(server_trigger.target()) {
        let child_entity = commands.spawn(
            AttackMarker,
        ).insert(ChildOf(server_trigger.target()))
        .id();

        active_skills.0.insert(server_trigger.attack_id, child_entity);

        attack_reg.spawn(server_trigger.attack_id, &mut commands, child_entity, server_trigger.offset);
    }
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
    pub owner: Entity,
    pub island: u64,
    pub offset: IVec3,
    pub damage: u64
}

impl DamageEvent {
    pub fn new(owner: Entity, island: u64, offset: IVec3, damage: u64) -> Self {
        Self { owner, island, offset, damage }
    }
}

#[derive(Event)]
pub struct NegatedDamageEvent {
    pub owner: Entity,
    pub victim: Entity,
    pub island: u64,
    pub offset: IVec3,
    pub damage: u64,
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

    commands.entity(damage_trigger.target()).insert(DamageVisualizer {
        timer: Timer::from_seconds(0.1, TimerMode::Once),
        flash_color: Color::WHITE,
        original_color: None,
    });

    if damage_trigger.remaining_health == 0 {
        if let Ok(_enemy_entity) = enemies.get(damage_trigger.target()) {
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
    mut health: Query<(&mut Health, Option<&Children>)>,
    negate_query: Query<&NegatingDamage>,
    server: Option<Res<RenetServer>>,
    mut commands: Commands
) {
    if server.is_some() {
        if let Some(map) = island_maps.maps.get(&damage_trigger.island) {
            if let Some(victim) = map.get_target(damage_trigger.offset) {
                if let Ok((mut hp, children)) = health.get_mut(victim) {
                    println!("Victim found");
                    let mut negated = false;

                    if let Some(children) = children { // checking if there is any ability negating the attack
                        for child in children.iter() {
                            if let Ok(negate_instance) = negate_query.get(child) {

                                negated = true;
                                commands.server_trigger(ToClients { 
                                    mode: SendMode::Broadcast, 
                                    event: NegateDamageTrigger {
                                        attack_id: negate_instance.0,
                                        owner: damage_trigger.owner,
                                        victim: victim,
                                        island: damage_trigger.island,
                                        offset: damage_trigger.offset,
                                        damage: damage_trigger.damage,
                                    }},
                                );
                                break;
                            }
                        }
                    }
                    
                    if !negated {
                        let remaining_health = hp.damage(damage_trigger.damage);
                        println!("doing the damage: {}", remaining_health);
                        commands.server_trigger_targets(
                            ToClients {
                                mode: SendMode::Broadcast,
                                event: ClientDamageEvent {
                                    amount: damage_trigger.damage,
                                    position: damage_trigger.offset,
                                    remaining_health,
                                },
                            },
                            victim,
                        );
                    }
                    
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
    mut cooldowns_query: Query<&mut AttackCooldowns>,
    mut active_skills_q: Query<&mut ActiveSkills>,
) {
    if let Ok(cooldowns) = &mut cooldowns_query.get_mut(attack_trigger.entity) {
        if let Some(timer) = cooldowns.0.get_mut(&attack_trigger.attack_id) {
            if !timer.finished() { 
                return; 
            }
        }

        cooldowns.0.insert(attack_trigger.attack_id, Timer::from_seconds(attack_cat.0.get(&attack_trigger.attack_id).unwrap().cooldown, TimerMode::Once));
    }

    if let Ok(mut active_skills) = active_skills_q.get_mut(attack_trigger.entity) {
        let child_entity = commands.spawn(
            AttackMarker,
        ).insert(ChildOf(attack_trigger.entity))
        .id();

        active_skills.0.insert(attack_trigger.attack_id, child_entity);

        commands.client_trigger_targets(
            ClientAttack {
                attack_id: attack_trigger.attack_id,
                offset: attack_trigger.offset
            },
            attack_trigger.entity
        );

        attack_reg.spawn(attack_trigger.attack_id, &mut commands, child_entity, attack_trigger.offset);
    }
}

#[derive(Component, Default)]
pub struct Interruptable;

#[derive(Component, Default)]
pub struct NegatingDamage(pub u64);

fn interrupt_attack_stun(
    mut commands: Commands,
    attacks: Query<(Entity, &ChildOf), With<Interruptable>>,
    stunned: Query<(), With<Stunned>>,
) {
    for (attack_entity, parent) in &attacks {
        if stunned.get(parent.0).is_ok() {
            commands.entity(attack_entity).despawn();
        }
    }
}

fn damage_visualizer_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageVisualizer, &VisualRef)>,
    mut visuals: Query<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<VisualEntity>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut visualizer, visual_ref) in &mut query {
        if let Ok((mut transform, material_handle)) = visuals.get_mut(**visual_ref) {
            visualizer.timer.tick(time.delta());

            let shake_intensity = 0.1;
            transform.translation.x += (rand::random::<f32>() - 0.5) * shake_intensity;
            transform.translation.y += (rand::random::<f32>() - 0.5) * shake_intensity;

            if let Some(material) = materials.get_mut(material_handle.id()) {
                if visualizer.original_color.is_none() {
                    visualizer.original_color = Some(material.base_color);
                }

                material.base_color = visualizer.flash_color;
            }

            if visualizer.timer.finished() {
                if let Some(original) = visualizer.original_color {
                    if let Some(material) = materials.get_mut(material_handle.id()) {
                        material.base_color = original;
                    }
                }

                transform.translation.x = transform.translation.x.round();
                transform.translation.y = transform.translation.y.round();

                commands.entity(entity).remove::<DamageVisualizer>();
            }
        }
    }
}