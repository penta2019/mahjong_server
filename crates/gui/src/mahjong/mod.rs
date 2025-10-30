mod action;
mod dialog;
mod has_entity;
mod model;
mod param;
mod plugin;
mod prelude;
mod setting;
mod text;
mod tile_plugin;

#[allow(unused)]
mod plugin_dev;

#[cfg(not(feature = "dev"))]
pub type MahjongPlugin = plugin::MahjongPlugin;
#[cfg(feature = "dev")]
pub type MahjongPlugin = plugin_dev::MahjongPlugin; // cargo run --release --features gui_dev G

pub use self::plugin::{Rx, Tx};
