use bevy::prelude::*;

pub mod objects;
pub mod preludes;
pub mod plugins;

pub const CHUNK_SIZE : i32 = 16;

use plugins::network::NetworkPlugin;
use plugins::enemy::EnemyPlugin;
use plugins::player::PlayerPlugin;
use plugins::camera::CameraPlugin;
use plugins::humanoid::HumanoidPlugin;

pub struct AppPlugin;
impl Plugin for AppPlugin {
  fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins, 
            NetworkPlugin, 
            PlayerPlugin,
            CameraPlugin,
            EnemyPlugin,    
            HumanoidPlugin
        )); 
    }
}