use bevy::prelude::*;
use crate::components::humanoid::ActionState;
use crate::components::island::OnIsland;
use crate::plugins::attack::{key_of, AttackCatalogue, AttackRegistry, AttackSpec, DamageEvent};
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::STANDARD;

const DAMAGE: u64 = 10;

#[derive(Component)]
pub struct BaseAttack {
    direction: IVec3,
    timer: Timer,
    hit: bool
}

impl Default for BaseAttack {
    fn default() -> Self {
        BaseAttack { 
            direction: IVec3::X, 
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            hit: false
        }
    }
}

pub struct BaseAttackPlugin;
impl Plugin for BaseAttackPlugin {
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
    registry.register::<BaseAttack>(|commands, entity, offset| {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.insert(BaseAttack {
                direction: offset,
                timer: Timer::from_seconds(0.20, TimerMode::Once),
                hit: false,
            });
        } 
    });
    let key = key_of::<BaseAttack>();
    catalog.0.insert(key, AttackSpec {offsets: &STANDARD, cooldown: 0.8, damage: DAMAGE });
}

fn perform_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut attacks: Query<(Entity, &Position, &mut Transform, &mut BaseAttack, &mut ActionState, &OnIsland)>,
) {
    for (entity, pos, mut transform, mut attack, mut state, island) in &mut attacks {
        *state = ActionState::Attacking;
        attack.timer.tick(time.delta());

        let t = (attack.timer.elapsed_secs() / attack.timer.duration().as_secs_f32()).clamp(0.0, 1.0);
        let magnitude = if t < 0.5 {
            t
        } else {
            1.0 - t
        };
        if !attack.hit && t >= 0.5 {
            attack.hit = true;
            commands.trigger(DamageEvent::new(
                island.0,
                pos.0 + attack.direction,
                DAMAGE
            ));
        }

        transform.translation = pos.0.as_vec3() + attack.direction.as_vec3() * magnitude * 0.5;

        if attack.timer.finished() {
            transform.translation = pos.0.as_vec3();
            commands.entity(entity).remove::<BaseAttack>();
            *state = ActionState::Idle;
        }
    }
}