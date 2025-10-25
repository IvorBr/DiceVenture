use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::island::*;
use crate::components::island_maps::Grid;
use crate::components::overworld::Enterable;
use crate::components::overworld::Entered;
use crate::components::overworld::LocalIsland;
use crate::islands::atoll::AtollPlugin;
use crate::islands::ship_map::ShipMapPlugin;
use crate::plugins::terrain_registry::TerrainRegistry;
use crate::plugins::network::MakeLocal;
use crate::components::character::LocalPlayer;
use crate::plugins::camera::NewCameraTarget;
use crate::components::character::Character;
use crate::plugins::network::OwnedBy;
use crate::preludes::network_preludes::*;
use crate::IslandSet;
use crate::GameState;
use bevy_replicon::prelude::ClientId::Server;

pub struct IslandPlugin;
impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((AtollPlugin, ShipMapPlugin))
        .add_mapped_client_message::<EnteredIsland>(Channel::Unordered)
        .add_server_message::<LeaveIsland>(Channel::Unordered)
        .replicate::<Character>()
        .replicate::<Waiting>()
        .add_systems(OnExit(GameState::Island), client_island_cleanup)
        .add_systems(PreUpdate, ((clean_up_island, add_waiting_player).run_if(in_state(ServerState::Running)), visualize_island))
        .add_systems(Update, (
            (player_enters_island, player_leaves_island, elimination_island_objective, chest_death_check).run_if(in_state(ServerState::Running)),
            (spawn_player_island, client_player_leaves_island).in_set(IslandSet),
            entered_island
        ));
    }
}

fn entered_island(
    mut commands: Commands,
    entered_query: Query<Entity, (Added<Entered>, With<Enterable>)>,
    mut state: ResMut<NextState<GameState>>,
    mut player_enter_event: MessageWriter<EnteredIsland>
) {
    for island_entity in entered_query.iter() {
        println!("ENTERING ISLAND PLAYER");
        commands.entity(island_entity).insert((LocalIsland, GenerateIsland, VisualizeIsland)).remove::<Entered>();
        player_enter_event.write(EnteredIsland(island_entity));
        state.set(GameState::Island);
    }
}

fn client_island_cleanup(
    mut commands: Commands,
    islandroot_query: Query<Entity, With<IslandRoot>>,
    island_entities_query: Query<Entity, With<VisualEntity>>,
) {
    //clean up visuals
    if let Ok(island_root) = islandroot_query.single() {
        println!("IslandRoot: {}", island_root);
        commands.entity(island_root).despawn();
    }

    //clean up entities
    for entity in island_entities_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn_player_island(
    mut commands: Commands,
    players: Query<(Entity, &Position, Option<&LocalPlayer>, &OnIsland), (With<Character>, Without<Transform>, Without<Waiting>)>,
    local_island_query: Query<Entity, With<LocalIsland>>,
    assets: Res<AssetServer>,
) {
    for (entity, position, local, island) in players.iter() {
        if let Ok(island_entity) = local_island_query.get(island.0) {
            println!("POS: {}", position.0);
            let scene: Handle<Scene> = assets.load("characters/BaseCharacter.glb#Scene0");
            let visual = commands.spawn((
                SceneRoot(scene),
                VisualEntity,
                Visibility::default(),
                AnimationPlayer::default()
            )).id();

            commands.entity(entity)
                .add_child(visual)
                .insert(VisualRef(visual))
                .insert(ChildOf(island_entity))
                .insert(
                    Transform::from_translation(position.0.as_vec3()).with_scale(Vec3::new(0.7, 0.7, 0.7)),
                );
            
            if local.is_some() { 
                commands.entity(entity).insert(NewCameraTarget);
            }
        }
    }
}

fn visualize_island(
    mut commands: Commands,
    islands: Query<(Entity, &Grid), (Without<GenerateIsland>, With<VisualizeIsland>)>,
    registry: Res<TerrainRegistry>,
){
    if let Ok((entity, grid)) = islands.single() {
        let island_visual_root = commands
        .spawn((
            IslandRoot,
            Transform::default(),
            InheritedVisibility::VISIBLE,
            ChildOf(entity)
        )).id();

        let mut to_spawn = Vec::new();

        for (pos, chunk) in grid.chunks.iter() {
            for (idx, tile) in chunk.tiles.iter().enumerate() {
                if let TileType::Terrain(tt) = tile.kind {
                    if let Some(info) = registry.get(&tt) {
                        let position = grid.chunk_to_world_coords(*pos, idx).as_vec3();
                        to_spawn.push((
                            SceneRoot(info.scene.clone()),
                            Transform::from_translation(position),
                            ChildOf(island_visual_root)
                        ));
                    }
                }
            }
        }

        commands.spawn_batch(to_spawn);

        commands.entity(entity).remove::<VisualizeIsland>();
    }
}  

fn add_waiting_player(
    mut commands: Commands,
    players: Query<(Entity, &OnIsland), With<Waiting>>,
    mut grid_query: Query<&mut Grid, With<FinishedSetupIsland>>,
    position_query: Query<&Position>
) {
    for (player_entity, island) in players.iter() {
        if let Ok(mut map) = grid_query.get_mut(island.0) {
            let mut spawn_pos = map.leave_position;
            spawn_pos.y += 2;
            while map.get_tile(spawn_pos).kind != TileType::Empty {
                spawn_pos.z += 1;
            }

            commands.entity(player_entity).insert(Position::new(spawn_pos)).remove::<Waiting>();

            commands.server_trigger(
                ToClients {
                    mode: SendMode::BroadcastExcept(Server),
                    message: ServerPositionUpdate { position: spawn_pos, entity: player_entity } ,
                },
            );

            for entity in map.entities.iter() {
                if let Ok(position) = position_query.get(*entity) {
                    commands.server_trigger(
                        ToClients {
                            mode: SendMode::BroadcastExcept(Server),
                            message: ServerPositionUpdate { position: position.0, entity: *entity} ,
                        },
                    );
                }   
            }

            map.add_player(spawn_pos, player_entity);
        }
    }
}

fn player_enters_island(
    mut commands: Commands,
    mut island_enter_event: MessageReader<FromClient<EnteredIsland>>,
) {
    for FromClient { client_id, message } in island_enter_event.read() {
        let island_entity = message.0;
        let player_entity = commands.spawn((
            Character,
            Health::new(200),
            Damage { value: 20 },
            OwnedBy(*client_id),
            Waiting,
            OnIsland(island_entity),
        )).id();
        
        commands.server_trigger(
            ToClients {
                mode: SendMode::Direct(*client_id),
                message: MakeLocal(player_entity),
            }
        );

        if *client_id != Server {
            commands.entity(island_entity).insert(GenerateIsland);
        }
        println!("ADDING PLAYER {:?}, WAITING FOR ISLAND", player_entity);
    }
}

fn elimination_island_objective(
    mut commands: Commands,
    mut target_query: Query<(Entity, &mut Grid), (With<FinishedSetupIsland>, With<EliminationObjective>, Without<CompletedIslandObjective>)>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    islandroot_query: Query<Entity, With<IslandRoot>>,
) {
    for (entity, mut map) in target_query.iter_mut() {
        if map.enemy_count == 0 {
            if let Ok(island_root) = islandroot_query.single() {
                let top_tiles = map.above_water_top_tiles();
                let chest_pos = top_tiles.choose(&mut rand::rng()).unwrap().clone() + IVec3::Y;

                println!("{} spawning chest", chest_pos);
                let chest_entity = commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.05, 1.05, 1.05))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 1.0),
                        ..Default::default()
                    })),
                    Transform::from_xyz(chest_pos.x as f32, chest_pos.y as f32, chest_pos.z as f32),
                    Position::new(chest_pos),
                    Chest,
                    Health::new(30),
                    OnIsland(entity),
                )).insert(ChildOf(island_root)).id();

                map.add_entity_ivec3(chest_pos, Tile::new(TileType::Enemy, chest_entity));
                commands.entity(entity).insert(CompletedIslandObjective);
            }
        }
    }
}

fn client_player_leaves_island(
    mut state: ResMut<NextState<GameState>>,
    mut leave_island_event: MessageReader<LeaveIsland>,
    mut grid_query: Query<&mut Grid, With<LocalIsland>>,
    client: Option<Res<RenetClient>>
) {
    for _ in leave_island_event.read() {
        if client.is_some() {
            if let Ok(mut grid) = grid_query.single_mut() {
                grid.reset();
            }
            println!("Deleting island on client");
        }
        state.set(GameState::Overworld);
    }
}

fn player_leaves_island(
    mut commands: Commands,
    player_query: Query<(&Position, Entity, &OwnedBy, &OnIsland), With<Character>>,
    mut grid_query: Query<&mut Grid>,
    mut leave_island_event: MessageWriter<ToClients<LeaveIsland>>,
) {
    for (position, entity, owner, island) in &player_query {
        if let Ok(mut map) = grid_query.get_mut(island.0) {
            if position.0 == map.leave_position + IVec3::Y {
                println!("{:?} leaves island", entity);

                commands.entity(entity).despawn();
                map.remove_entity(position.0);
                map.entities.remove(&entity);
                map.player_count -= 1;

                leave_island_event.write(ToClients { mode: SendMode::Direct(owner.0), message: LeaveIsland });
            }
        }
    }
}

fn clean_up_island( //TODO: terribly inefficient, to loop over all islands, could better check when player leaves if no player is left
    mut commands: Commands,
    players: Query<&OnIsland, Without<Enemy>>,
    mut grid_query: Query<(Entity, &mut Grid), With<FinishedSetupIsland>>
) {
    let mut grids: HashSet<Entity> = grid_query.iter().map(|(e, _)| e).collect();
    for island in players.iter() {
        grids.remove(&island.0);
    }

    for grid_entity in grids.iter() {
        println!("No players left on island {:?}: cleaning up", grid_entity);
        if let Ok((_, mut grid)) = grid_query.get_mut(*grid_entity) {
            for enemy_entity in grid.entities.iter() {
                commands.entity(*enemy_entity).insert(RemoveEntity);
            }
            grid.reset();
        }
        
        commands.entity(*grid_entity).remove::<FinishedSetupIsland>(); //TODO: Improve this logic, FinishedSetupIsland indicates that the map was finished loading, so we remove it so it can be loaded again...
    }
}   

fn chest_death_check(
    mut commands: Commands,
    entities: Query<(&Health, Entity), With<Chest>>,
) {
    for (health, entity) in entities.iter() {
        if health.get() == 0 {
            commands.entity(entity).insert(RemoveEntity);
            //reward players
        }
    }
}