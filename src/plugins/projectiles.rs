use bevy::prelude::*;

use crate::components::island_maps::{IslandMaps, TileType};
use crate::components::island::OnIsland;
use crate::plugins::attack::DamageEvent;

pub struct ProjectilePlugin;
impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(PreUpdate, (projectile_system));
    }
}

#[derive(Component)]
pub struct Projectile {
    pub owner: Entity,
    pub direction: Vec3,
    pub traveled: f32,
    pub range: u8,
    pub speed: f32,
    pub damage: u64,
}

fn projectile_system(
    time: Res<Time>,
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile, &OnIsland)>,
    island_maps: Res<IslandMaps>,
) {
    for (entity, mut transform, mut projectile, island) in &mut projectiles {
        let delta = projectile.speed * time.delta_secs();
        projectile.traveled += delta;

        transform.translation += projectile.direction * delta;

        if let Some(map) = island_maps.get_map(island.0) {
            let tile_pos = IVec3::new(transform.translation.x.round() as i32, transform.translation.y.round() as i32, transform.translation.z.round() as i32);
            let tile = map.get_tile(tile_pos);

            match tile.kind {
                TileType::Terrain(_) => {
                    commands.entity(entity).despawn();
                    continue;
                },
                TileType::Player | TileType::Enemy => {
                    if tile.entity != projectile.owner {
                        commands.trigger(DamageEvent::new(
                            projectile.owner,
                            island.0,
                            tile_pos,
                            projectile.damage
                        ));
                        commands.entity(entity).despawn();
                    }
                }
                _ => (),
            }

            if projectile.traveled >= projectile.range as f32 {
                commands.entity(entity).despawn(); //TODO: despawn animation
                continue;
            }
        }
    }
}
