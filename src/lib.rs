// src/lib.rs
// 31-Aug-2023

use bracket_lib::prelude::*;
use specs::prelude::*;
extern crate serde;

#[macro_use]
extern crate lazy_static;

pub mod camera;
pub mod components;
pub mod raws;
pub mod map;
pub mod player;
pub mod gamelog;
pub mod gui;
pub mod map_builders;
pub mod saveload_system;
pub mod spawner;
pub mod visibility_system;
pub mod damage_system;
pub mod hunger_system;
pub mod melee_combat_system;
pub mod trigger_system;
pub mod inventory;
pub mod particle_system;
pub mod ai;
pub mod consts;
pub mod config;
pub mod effects;
pub mod gamesystem;
pub mod random_table;
pub mod rex_assets;
pub mod spatial;
pub mod morgue;
pub mod states;

pub use components::*;
use particle_system::ParticleBuilder;
pub use map::*;
pub use states::runstate::RunState;
pub use states::state::State;
pub use states::state::Fonts;
