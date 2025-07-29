use bevy::math::ops::floor;
use bevy::prelude::*;
use crate::components::humanoid::ActionState;
use crate::components::island::OnIsland;
use crate::components::island_maps::{self, IslandMaps};
use crate::plugins::attack::{key_of, AttackCatalogue, AttackRegistry, AttackSpec, PreDamageEvent, Interruptable, Projectile};
use crate::preludes::humanoid_preludes::*;
use crate::components::enemy::STANDARD;

const DAMAGE: u64 = 8;
const ATTACK_RANGE : u8 = 3;


#[derive(Component)]
#[require(Interruptable)]
pub struct DaggerThrow {
    direction: IVec3,
    timer: Timer,
    hit: bool
}

impl Default for DaggerThrow {
    fn default() -> Self {
        DaggerThrow { 
            direction: IVec3::X, 
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            hit: false
        }
    }
}

pub struct DaggerThrowPlugin;
impl Plugin for DaggerThrowPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, register_attack)
        .add_systems(Update, perform_attack);
    }
}

fn register_attack(
    mut registry: ResMut<AttackRegistry>,
    mut catalog: ResMut<AttackCatalogue>,
) {
    registry.register::<DaggerThrow>(|commands, entity, offset| {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.insert(DaggerThrow {
                direction: offset,
                timer: Timer::from_seconds(0.20, TimerMode::Once),
                hit: false,
            });
        }
    });
    let key = key_of::<DaggerThrow>();
    catalog.0.insert(key, AttackSpec {offsets: &STANDARD, cooldown: 0.8, damage: DAMAGE });
}

fn perform_attack(
    time: Res<Time>,
    mut commands: Commands,
    island_maps: Res<IslandMaps>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut attacks: Query<(Entity, &ChildOf, &mut DaggerThrow)>,
    mut parent_query: Query<(&Position, &mut Transform, &mut ActionState, &OnIsland)>,
) {
        for (child_entity, parent, mut attack) in &mut attacks {
        if let Ok((pos, mut transform, mut state, island)) = parent_query.get_mut(parent.0) {
            *state = ActionState::Attacking;
            attack.timer.tick(time.delta());

            if island_maps.get_map(island.0).is_some() && !attack.hit {
                attack.hit = true;
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.3, 0.3, 0.3))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 1.0),
                        ..Default::default()
                    })),
                    Projectile {
                        owner: parent.0,
                        traveled: 0.0,
                        range: ATTACK_RANGE,
                        direction: Vec3::new(attack.direction.x as f32, attack.direction.y as f32, attack.direction.z as f32),
                        speed: 1.0,
                        damage: DAMAGE
                    },
                    Transform::from_translation(transform.translation),
                    OnIsland(island.0)
                ));
            }

            if attack.timer.finished() {
                transform.translation = pos.0.as_vec3();
                commands.entity(child_entity).despawn();
                *state = ActionState::Idle;
            }
        }
    }
}