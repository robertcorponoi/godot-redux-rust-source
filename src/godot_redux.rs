use godot::prelude::*;
use godot::classes::Object;
use godot::builtin::{Callable, Dictionary, StringName};

#[derive(GodotClass)]
#[class(init, base = Object)]
pub struct GodotRedux {
    #[base]
    base: Base<Object>,

    /// The current state of the application.
    #[init(val = Dictionary::new())]
    state: Dictionary,

    /// The reducer function (object + method).
    #[init(val = Callable::invalid())]
    reducer: Callable,

    /// Middleware chain.
    #[init(val = Vec::new())]
    middleware: Vec<Callable>,

    /// Subscriptions called after state changes.
    #[init(val = Vec::new())]
    subscriptions: Vec<Callable>,
}

#[godot_api]
impl GodotRedux {
    /// Creates/overwrites the store: sets state and reducer.
    ///
    /// - `reducer_instance`: the object that has the reducer method
    /// - `reducer_method`: method name as a StringName (by value)
    #[func]
    fn set_state_and_reducer(
        &mut self,
        initial_state: Dictionary,
        reducer_instance: Gd<Object>,
        reducer_method: StringName, // PASS BY VALUE
    ) {
        self.state = initial_state;
        self.reducer = Callable::from_object_method(&reducer_instance, reducer_method);
        self.middleware.clear();
        self.subscriptions.clear();
    }

    /// Returns the current state (shared handle).
    #[func]
    fn state(&self) -> Dictionary {
        self.state.clone()
        // If you want a copy:
        // self.state.duplicate_shallow()
    }

    /// Dispatches an action to update the state.
    #[func]
    fn dispatch(&mut self, action: i64) {
        if self.middleware.is_empty() {
            self.dispatch_reducer(action);
        } else {
            self.dispatch_middleware(0, action);
        }
    }

    /// Subscribes to state changes: `subscriber_instance.subscriber_method(state)`
    #[func]
    fn subscribe(&mut self, subscriber_instance: Gd<Object>, subscriber_method: StringName) {
        let callable = Callable::from_object_method(&subscriber_instance, subscriber_method); // PASS BY VALUE
        self.subscriptions.push(callable);
    }

    /// Adds a middleware: `middleware_instance.middleware_method(state, action) -> i64 | Nil`
    #[func]
    fn add_middleware(&mut self, middleware_instance: Gd<Object>, middleware_method: StringName) {
        let callable = Callable::from_object_method(&middleware_instance, middleware_method); // PASS BY VALUE
        self.middleware.push(callable);
    }
}

// -------- internals (not exported) --------
impl GodotRedux {
    fn dispatch_middleware(&mut self, index: usize, action: i64) {
        if index == self.middleware.len() {
            self.dispatch_reducer(action);
            return;
        }

        let args = &[self.state.to_variant(), action.to_variant()];
        let next_variant = self.middleware[index].call(args);
        let next_action = next_variant.try_to_relaxed::<i64>().ok();

        if let Some(x) = next_action {
            self.dispatch_middleware(index + 1, x);
        }
    }

    fn dispatch_reducer(&mut self, action: i64) {
        let args = &[self.state.to_variant(), action.to_variant()];
        let new_state_v = self.reducer.call(args);

        // Expect reducer to return a Dictionary; default to empty on mismatch.
        self.state = new_state_v
            .try_to::<Dictionary>()
            .unwrap_or_else(|_| Dictionary::new());

        self.dispatch_subscriptions();
    }

    fn dispatch_subscriptions(&self) {
        let args = &[self.state.to_variant()];
        for sub in &self.subscriptions {
            sub.call(args);
        }
    }
}

