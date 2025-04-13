use bevy::prelude::*;

pub mod components;
pub mod preludes;
pub mod plugins;

use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use plugins::island::IslandPlugin;
use plugins::network::NetworkPlugin;

use plugins::camera::CameraPlugin;

use plugins::humanoid::HumanoidPlugin;
use plugins::enemy::EnemyPlugin;
use plugins::island_controls::PlayerPlugin;

use plugins::overworld::OverworldPlugin;
use plugins::ship::ShipPlugin;

#[derive(States, PartialEq, Eq, Debug, Hash, Clone)]
enum GameState {
    Initializing,
    Overworld,
    Island,
}

pub const CHUNK_SIZE : i32 = 16;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct IslandSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct OverworldSet;

pub struct AppPlugin;
impl Plugin for AppPlugin {
  fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
            backends: Some(Backends::VULKAN),
            ..default()
        }),
        ..default()
        }))
        .insert_state(GameState::Initializing)
        .configure_sets(Update, (
            IslandSet.run_if(in_state(GameState::Island)),
        ))
        .configure_sets(PreUpdate, (
            IslandSet.run_if(in_state(GameState::Island)),
        ))
        .configure_sets(Update, (
            OverworldSet.run_if(in_state(GameState::Overworld)),
        ))
        .add_plugins((
            NetworkPlugin,
            CameraPlugin,
            
            OverworldPlugin,
            ShipPlugin,

            IslandPlugin,
            PlayerPlugin,
            EnemyPlugin,    
            HumanoidPlugin,
        ));
    }
}