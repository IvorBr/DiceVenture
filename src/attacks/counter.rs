use bevy::prelude::*;
use crate::components::humanoid::{ActionState, Stunned};
use crate::plugins::attack::{key_of, AttackCatalogue, AttackRegistry, AttackSpec, NegatingDamage, NegatedDamageEvent};
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::STANDARD;

const DAMAGE: u64 = 10;
const ATTACK_LENGTH: f32 = 5.0;
const COOLDOWN: f32 = 6.0;

#[derive(Component)]
#[require(NegatingDamage(key_of::<Counter>()))]
pub struct Counter {
    direction: IVec3,
    timer: Timer,
    hit: bool
}


impl Default for Counter {
    fn default() -> Self {
        Counter { 
            direction: IVec3::X, 
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            hit: false
        }
    }
}

pub struct CounterPlugin;
impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, register_base_attack)
        .add_systems(Update, (perform_attack, process_counter));
    }
}

fn register_base_attack(
    mut registry: ResMut<AttackRegistry>,
    mut catalog: ResMut<AttackCatalogue>,
) {
    registry.register::<Counter>(|commands, entity, offset| {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.insert(Counter {
                direction: offset,
                timer: Timer::from_seconds(ATTACK_LENGTH, TimerMode::Once),
                hit: false,
            });
        } 
    });
    let key = key_of::<Counter>();
    catalog.0.insert(key, AttackSpec {offsets: &STANDARD, cooldown: COOLDOWN, damage: DAMAGE });
}

fn process_counter(
    mut reader: EventReader<NegatedDamageEvent>,
    counter_query: Query<Entity, With<Counter>>,
    mut commands: Commands,
) {
    for event in reader.read() {
        if let Ok(entity) = counter_query.get(event.victim) {
            commands.entity(event.owner).insert(Stunned::new(10.0));
            commands.entity(entity).despawn();
        }
    }
}

fn perform_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut attacks: Query<(Entity, &ChildOf, &mut Counter)>,
    mut parent_query: Query<(&Position, &mut Transform, &mut ActionState)>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (child_entity, parent, mut attack) in &mut attacks {
        if let Ok((pos, mut transform, mut state)) = parent_query.get_mut(parent.0) {
            *state = ActionState::Attacking;
            if attack.timer.elapsed_secs() == 0.0 {
                commands.entity(child_entity).insert((
                        Mesh3d(meshes.add(Cuboid::new(1.1, 1.1, 1.1))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.5, 1.0, 0.5),
                            ..Default::default()
                        })),
                        Transform::from_xyz(0.0, 0.0, 0.0)
                    ));
            }
            attack.timer.tick(time.delta());

            if attack.timer.finished() {
                transform.translation = pos.0.as_vec3();
                commands.entity(child_entity).despawn();
                *state = ActionState::Idle;
            }
        }
    }
}