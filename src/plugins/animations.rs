use bevy::prelude::*;

#[derive(Resource)]
pub struct IdleGraph {
    pub handle: Handle<AnimationGraph>,
    pub node: AnimationNodeIndex
}

pub struct AnimationsPlugin;
impl Plugin for AnimationsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, build_idle_graph)
        .add_systems(Update, attach_idle_to_new_players);
    }
}

fn build_idle_graph(
    mut graphs: ResMut<Assets<AnimationGraph>>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let clip: Handle<AnimationClip> = assets.load("characters/BaseCharacter.glb#Animation0");

    let (graph, node) = AnimationGraph::from_clip(clip.clone());

    commands.insert_resource(IdleGraph{ handle: graphs.add(graph), node: node});
}

fn attach_idle_to_new_players(
    mut q: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    idle: Res<IdleGraph>,
    mut commands: Commands
) {
    for (entity, mut player) in &mut q {
        commands.entity(entity).insert(AnimationGraphHandle(idle.handle.clone()));
        player.play(idle.node).repeat();
    }
}