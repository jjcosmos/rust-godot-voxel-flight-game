use godot::prelude::*;

mod cube_spawner;
mod demo;
mod player;
mod player_cam;
struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
