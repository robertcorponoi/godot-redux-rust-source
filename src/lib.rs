// src/lib.rs

mod godot_redux; // keep your module

use godot::prelude::*;

// v0.4+ GDExtension entrypoint. No InitHandle or godot_init! macro.
struct GodotReduxLib;

#[gdextension]
unsafe impl ExtensionLibrary for GodotReduxLib {}
