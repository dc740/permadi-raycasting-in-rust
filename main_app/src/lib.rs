extern crate farfarbfeld;

pub mod game;
pub mod loader;

mod generic_loader_impl;

#[cfg(feature = "web")]
pub mod web_setup;
