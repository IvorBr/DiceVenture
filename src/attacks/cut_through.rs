use bevy::prelude::*;
use crate::components::humanoid::ActionState;
use crate::components::island::OnIsland;
use crate::components::island_maps::{self, IslandMaps, Tile, TileType};
use crate::plugins::attack::{key_of, AttackCatalogue, AttackRegistry, AttackSpec, DamageEvent};
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::STANDARD;

const DAMAGE: u64 = 30;

#[derive(Component)]
pub struct CutThrough {
    direction: IVec3,
    timer: Timer,
    hit: bool
}

impl Default for CutThrough {
    fn default() -> Self {
        CutThrough { 
            direction: IVec3::X, 
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            hit: false
        }
    }
}

pub struct CutThroughPlugin;
impl Plugin for CutThroughPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, register_base_attack)
        .add_systems(Update, perform_attack);
    }
}

fn register_base_attack(
    mut registry: ResMut<AttackRegistry>,
    mut catalog: ResMut<AttackCatalogue>,
) {
    registry.register::<CutThrough>(|commands, entity, offset| {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.insert(CutThrough {
                direction: offset,
                timer: Timer::from_seconds(0.00, TimerMode::Once),
                hit: false,
            });
        } 
    });
    let key = key_of::<CutThrough>();
    catalog.0.insert(key, AttackSpec {offsets: &STANDARD, cooldown: 6.0, damage: DAMAGE });
}

fn perform_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut attacks: Query<(Entity, &mut Position, &mut Transform, &mut CutThrough, &mut ActionState, &OnIsland)>,
    mut island_maps: ResMut<IslandMaps>
) {
    for (entity, mut pos, mut transform, mut attack, mut state, island) in &mut attacks {
        *state = ActionState::Attacking;
        attack.timer.tick(time.delta());

        if attack.timer.finished() {
            let mut check_pos = pos.0 + attack.direction;
            if let Some(map) = island_maps.get_map_mut(island.0) {
                while map.get_target(check_pos).is_some() {
                    commands.trigger(DamageEvent::new(
                        island.0,
                        check_pos,
                        DAMAGE
                    ));

                    check_pos += attack.direction;
                }

                if map.can_move(check_pos) {
                    map.remove_entity(pos.0);
                    map.add_entity_ivec3(check_pos, Tile::new(TileType::Player, entity));
                    pos.0 = check_pos;
                }
            }

            transform.translation = pos.0.as_vec3();
            commands.entity(entity).remove::<CutThrough>();
            *state = ActionState::Idle;
        }
    }
}