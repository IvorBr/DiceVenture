use bevy::prelude::*;
use crate::components::humanoid::{ActionState, VisualEntity, VisualRef};
use crate::components::island::OnIsland;
use crate::plugins::attack::{key_of, AttackCatalogue, AttackRegistry, AttackSpec, DamageEvent, Interruptable};
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::STANDARD;

const DAMAGE: u64 = 10;

#[derive(Component)]
#[require(Interruptable)]
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
    catalog.0.insert(key, AttackSpec {offsets: &STANDARD, cooldown: 0.4, damage: DAMAGE });
}

fn perform_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut attacks: Query<(Entity, &ChildOf, &mut BaseAttack)>,
    mut parent_query: Query<(&Position, &mut ActionState, &OnIsland)>,
    visual_query: Query<&VisualRef>,
    mut transform_query: Query<(&mut Transform, &GlobalTransform), With<VisualEntity>>
) {
    for (child_entity, parent, mut attack) in &mut attacks {
        let mut t = 0.0;
        if let Ok((pos, mut state, island)) = parent_query.get_mut(parent.0) { //probably missing visual, globaltransform, or something when server is not on island...
            //logic
            *state = ActionState::Attacking;
            attack.timer.tick(time.delta());

            t = (attack.timer.elapsed_secs() / attack.timer.duration().as_secs_f32()).clamp(0.0, 1.0);
            if !attack.hit && t >= 0.5 {
                attack.hit = true;
                commands.trigger(DamageEvent::new(
                    parent.0,
                    island.0,
                    pos.get() + attack.direction,
                    DAMAGE
                ));
            }

            if attack.timer.finished() {
                commands.entity(child_entity).despawn();
                *state = ActionState::Idle;
            }
        }    

        //visual
        if let Ok(visual_ref) = visual_query.get(parent.0) {
            if let Ok((mut transform, global_transform)) = transform_query.get_mut(**visual_ref) {
                let magnitude = if t < 0.5 {
                    t
                } else {
                    1.0 - t
                };

                transform.translation = global_transform.rotation().inverse() * attack.direction.as_vec3() * magnitude * 0.5;
            }  
        }
    }
}