//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.

#[cfg(target_pointer_width = "32")]
compile_error!("This game is not supported on 32-bit systems.");

pub mod entities;
pub mod gui;
pub mod physics;
