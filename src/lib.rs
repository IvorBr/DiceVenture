use bevy::prelude::*;

pub mod objects;
pub mod preludes;
pub mod plugins;

use plugins::network::NetworkPlugin;

use plugins::camera::CameraPlugin;

use plugins::humanoid::HumanoidPlugin;
use plugins::enemy::EnemyPlugin;
use plugins::player::PlayerPlugin;

use plugins::overworld::OverworldPlugin;

#[derive(States, PartialEq, Eq, Debug, Hash, Clone)]
enum GameState {
    Overworld,
    InGame,
}

pub const CHUNK_SIZE : i32 = 16;

pub struct AppPlugin;
impl Plugin for AppPlugin {
  fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
        app.insert_state(GameState::Overworld);
        app.add_plugins((
            // NetworkPlugin, 
            // PlayerPlugin,
            CameraPlugin,
            // EnemyPlugin,    
            // HumanoidPlugin,
            OverworldPlugin
        )); 
    }
}