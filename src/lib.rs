mod godot_redux;
use gdnative::prelude::*;

fn init(handle: InitHandle) {
    handle.add_class::<godot_redux::GodotRedux>();
}

godot_init!(init);
