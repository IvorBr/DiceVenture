use bevy::prelude::*;

use crate::preludes::humanoid_preludes::*;
use crate::preludes::network_preludes::*;

pub struct HumanoidPlugin;
impl Plugin for HumanoidPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(PreUpdate,
            (
                death_check.run_if(server_running).before(remove_entities),
                (remove_entities).after(ClientSet::Receive),
                move_entities
            )
        );
    }
}

fn death_check(
    mut commands: Commands,
    entities: Query<(&Health, Entity), Or<(With<Player>, With<Enemy>)>>
) {
    for (health, entity) in &entities {
        if health.get() == 0 {
            println!("{}, {}", entity, health.get());
            commands.entity(entity).insert(RemoveEntity);
        }
    }
}

fn remove_entities(mut commands: Commands,
    entities: Query<(Entity, &Position), With<RemoveEntity>>,
    mut map: ResMut<Map>
) {
    for (entity, position) in &entities {
        map.remove_entity(position.0);
        println!("Despawning entity: {:?}", entity);
        commands.entity(entity).despawn();
    }
}

fn move_entities(
    mut moved_entities: Query<(&Position, &mut Transform)>, 
    time: Res<Time>
){
    for (position, mut transform) in &mut moved_entities {
        if position.0.as_vec3() != transform.translation {
            transform.translation = transform
                .translation
                .lerp(position.0.as_vec3(), time.delta_seconds() * 10.0);
        }
    }
}