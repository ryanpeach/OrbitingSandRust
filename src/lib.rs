//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.
#[warn(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::panic,
    clippy::trivially_copy_pass_by_ref,
    clippy::inefficient_to_string,
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_fields_in_debug,
    clippy::redundant_clone
)]
#[deny(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
#[allow(clippy::too_many_lines)]
pub mod entities;
pub mod gui;
pub mod physics;
