mod player;
mod terrain;

use crate::player::*;
use crate::terrain::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{TnuaPlatformerPlugin, TnuaRapier3dPlugin};

pub const SEED: f32 = 23.0;
pub const LANES: i32 = 10;
pub const MAX_OBSTACLES_PER_LANE: i32 = 100;
pub const LANES_WIDTH: f32 = 10.0;
pub const OBSTACLE_LENGTH: f32 = 50.0;
pub const OBSTACLE_HEIGHT: f32 = 3.0;
pub const MAX_OBSTACLE_STACK: i32 = 10;
pub const DEFAULT_POSITION: Vec3 = Vec3::Y;
pub const MOVE_COOLDOWN: f32 = 0.2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(TnuaRapier3dPlugin)
        .add_plugin(TnuaPlatformerPlugin)
        .add_startup_system(setup_terrain)
        .add_startup_system(setup_player)
        .add_system(player_movement)
        .add_system(check_wall_crashes)
        .add_system(animation_patcher_system)
        .add_system(player_animate)
        .run();
}
