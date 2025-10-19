use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::components::enemy::*;
use crate::components::humanoid::*;
use crate::components::island::*;
use crate::components::island_maps::Grid;
use crate::components::island_maps::TerrainType;
use crate::components::overworld::Entered;
use crate::components::overworld::{LocalIsland, Island};
use crate::islands::atoll::AtollPlugin;
use crate::islands::ship_map::ShipMapPlugin;
use crate::plugins::network::MakeLocal;
use crate::components::character::LocalPlayer;
use crate::plugins::camera::NewCameraTarget;
use crate::components::character::Character;
use crate::plugins::network::OwnedBy;
use crate::preludes::network_preludes::*;
use crate::IslandSet;
use crate::GameState;

pub struct IslandPlugin;
impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((AtollPlugin, ShipMapPlugin))
        .add_mapped_client_event::<EnteredIsland>(Channel::Unordered)
        .add_server_event::<LeaveIsland>(Channel::Unordered)
        .replicate::<Character>()
        .replicate::<Waiting>()
        .add_systems(OnExit(GameState::Island), client_island_cleanup)
        .add_systems(PreUpdate, ((clean_up_island, add_waiting_player).run_if(server_running), visualize_island))
        .add_systems(Update, (
            (player_enters_island, player_leaves_island, elimination_island_objective).run_if(server_running),
            (spawn_player_island, client_player_leaves_island).in_set(IslandSet),
            entered_island
        ));
    }
}

fn entered_island(
    mut commands: Commands,
    entered_query: Query<Entity, (Added<Entered>, With<Island>)>,
    mut state: ResMut<NextState<GameState>>,
    mut player_enter_event: EventWriter<EnteredIsland>
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
            commands.entity(entity).insert((
                Transform::from_translation(position.0.as_vec3()).with_scale(Vec3::new(0.7, 0.7, 0.7)),
            ));
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
                .insert(ChildOf(island_entity));
            
            if local.is_some() { 
                commands.entity(entity).insert(NewCameraTarget);
            }
        }
    }
}

fn visualize_island(
    mut commands: Commands,
    islands: Query<(Entity, &Grid, &Transform), (Without<GenerateIsland>, With<VisualizeIsland>)>,
    assets: Res<AssetServer>,
){
    if let Ok((entity, grid, island_position)) = islands.single() {
        let island_visual_root = commands
        .spawn((
            IslandRoot,
            Transform::from_translation(island_position.translation.round()),
            InheritedVisibility::VISIBLE
        )).id();

        for (pos, chunk) in grid.chunks.iter() {
            for (idx, tile) in chunk.tiles.iter().enumerate() {
                if !matches!(tile.kind, TileType::Terrain(_)) {
                    continue;
                }

                let position = grid.chunk_to_world_coords(*pos, idx);
                let mut mesh: Handle<Mesh> = Handle::default();
                let mut material: Handle<StandardMaterial> = Handle::default();

                match tile.kind {
                    TileType::Terrain(TerrainType::Sand) => {
                        mesh = assets.load("blocks/SandBlockTest.glb#Mesh0/Primitive0").clone();
                        material = assets.load("blocks/SandBlockTest.glb#Material0").clone();
                    }
                    TileType::Terrain(TerrainType::Rock) => {
                        mesh = assets.load("blocks/RockBlock.glb#Mesh0/Primitive0").clone();
                        material = assets.load("blocks/RockBlock.glb#Material0").clone();
                    }
                    TileType::Terrain(TerrainType::Boardwalk) => {
                        mesh = assets.load("blocks/BoardWalkBlock.glb#Mesh0/Primitive0").clone();
                        material = assets.load("blocks/BoardWalkBlock.glb#Material0").clone();
                    }
                    TileType::Terrain(TerrainType::PalmTree) => {
                        mesh = assets.load("blocks/PalmTreeTrunkBlock.glb#Mesh0/Primitive0").clone();
                        material = assets.load("blocks/PalmTreeTrunkBlock.glb#Material0").clone();
                    }
                    _ => ()
                }

                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(position.as_vec3()),
                )).insert(ChildOf(island_visual_root));
            }
        }

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
            println!("SETTING WAITING PLAYER TO: {}", map.leave_position);
            let mut spawn_pos = map.leave_position;
            spawn_pos.y += 2;
            while map.get_tile(spawn_pos).kind != TileType::Empty {
                spawn_pos.z += 1;
            }

            commands.entity(player_entity).insert(Position::new(spawn_pos)).remove::<Waiting>();

            commands.server_trigger_targets(
                ToClients {
                    mode: SendMode::BroadcastExcept(SERVER),
                    event: ServerPositionUpdate { position: spawn_pos } ,
                },
                player_entity,
            );

            for entity in map.entities.iter() {
                if let Ok(position) = position_query.get(*entity) {
                    commands.server_trigger_targets(
                        ToClients {
                            mode: SendMode::BroadcastExcept(SERVER),
                            event: ServerPositionUpdate { position: position.0 } ,
                        },
                        *entity,
                    );
                }   
            }

            map.add_player(spawn_pos, player_entity);
        }
    }
}

fn player_enters_island(
    mut commands: Commands,
    mut island_enter_event: EventReader<FromClient<EnteredIsland>>,
) {
    for FromClient { client_entity, event } in island_enter_event.read() {
        let island_entity = event.0;
        let player_entity = commands.spawn((
            Character,
            Health::new(200),
            Damage { value: 20 },
            OwnedBy(*client_entity),
            Waiting,
            OnIsland(island_entity),
        )).id();
        
        commands.server_trigger_targets(
            ToClients {
                mode: SendMode::Direct(*client_entity),
                event: MakeLocal,
            },
            player_entity,
        );

        if *client_entity != SERVER {
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
    mut leave_island_event: EventReader<LeaveIsland>,
    mut grid_query: Query<&mut Grid, With<LocalIsland>>,
    client: Option<Res<RenetClient>>
) {
    for event in leave_island_event.read() {
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
    island_query: Query<&Island>,
    mut grid_query: Query<&mut Grid>,
    mut leave_island_event: EventWriter<ToClients<LeaveIsland>>,
) {
    for (position, entity, owner, island) in &player_query {
        if let Ok(mut map) = grid_query.get_mut(island.0) {
            if position.0 == map.leave_position + IVec3::Y {
                println!("{:?} leaves island", entity);

                commands.entity(entity).despawn();
                map.remove_entity(position.0);
                map.entities.remove(&entity);
                map.player_count -= 1;

                leave_island_event.write(ToClients { mode: SendMode::Direct(owner.0), event: LeaveIsland });
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
        if let Ok((grid_entity, mut grid)) = grid_query.get_mut(*grid_entity) {
            for enemy_entity in grid.entities.iter() {
                commands.entity(*enemy_entity).insert(RemoveEntity);
            }
            grid.reset();
        }
        
        commands.entity(*grid_entity).remove::<(FinishedSetupIsland)>(); //TODO: Improve this logic, FinishedSetupIsland indicates that the map was finished loading, so we remove it so it can be loaded again...
    }
}   