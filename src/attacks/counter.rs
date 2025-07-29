use bevy::prelude::*;
use crate::components::humanoid::ActionState;
use crate::components::island::OnIsland;
use crate::plugins::attack::{key_of, AttackCatalogue, AttackRegistry, AttackSpec, PreDamageEvent};
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::STANDARD;

const DAMAGE: u64 = 10;

#[derive(Component)]
pub struct Counter {
    direction: IVec3,
    timer: Timer,
    hit: bool
}

#[derive(Component)]
pub struct CounterVisualMarker;

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
        .add_systems(Update, perform_attack);
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
                timer: Timer::from_seconds(0.20, TimerMode::Once),
                hit: false,
            });
        } 
    });
    let key = key_of::<Counter>();
    catalog.0.insert(key, AttackSpec {offsets: &STANDARD, cooldown: 0.8, damage: DAMAGE });
}

fn perform_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut attacks: Query<(Entity, &ChildOf, &mut Counter)>,
    mut parent_query: Query<(&Position, &mut Transform, &mut ActionState, &OnIsland, &Children)>,
    marker_query: Query<&CounterVisualMarker>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (child_entity, parent, mut attack) in &mut attacks {
        if let Ok((pos, mut transform, mut state, island, children)) = parent_query.get_mut(parent.0) {
            *state = ActionState::Attacking;
            if attack.timer.elapsed_secs() == 0.0 {

                commands.entity(parent.0).with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.1, 1.1, 1.1))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.5, 1.0, 0.5),
                            ..Default::default()
                        })),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                        CounterVisualMarker
                    ));
                });
            }
            attack.timer.tick(time.delta());

            if attack.timer.finished() {
                transform.translation = pos.0.as_vec3();
                commands.entity(child_entity).despawn();

                for child in children.iter() {
                if marker_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }

                *state = ActionState::Idle;
            }
        }
    }
}