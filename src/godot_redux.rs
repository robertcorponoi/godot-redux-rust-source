use gdnative::api::FuncRef;
use gdnative::prelude::{
    core_types::GodotString, methods, Dictionary, NativeClass, Object, Ref, Shared, Unique, Variant,
};

#[inherit(Object)]
#[derive(NativeClass)]
pub struct GodotRedux {
    /// The initial state of the application.
    state: Dictionary,
    /// The reducer function.
    reducer: Ref<FuncRef, Unique>,
    /// The middleware functions used to intercept actions and change them
    /// before they reach the reducer.
    middleware: Vec<Ref<FuncRef, Unique>>,
    /// The callback functions to run when the state is changed.
    subscriptions: Vec<Ref<FuncRef, Unique>>,
}

#[methods]
impl GodotRedux {
    /// Initializes the struct with default values.
    fn new(_owner: &Object) -> Self {
        GodotRedux {
            state: Dictionary::new_shared(),
            reducer: FuncRef::new(),
            middleware: vec![],
            subscriptions: vec![],
        }
    }

    /// Creates a new store
    ///
    /// # Arguments
    ///
    /// * `initial_state` - The initial state of the application.
    /// * `reducer_fn_instance` - The instance on which the reducer exists.
    /// * `reducer_fn_name` - The name of the reducer function.
    ///
    /// # Example
    ///
    ///
    #[export]
    fn set_state_and_reducer(
        &mut self,
        _owner: &Object,
        initial_state: Dictionary,
        reducer_fn_instance: Ref<Object, Shared>,
        reducer_fn_name: GodotString,
    ) {
        self.state = initial_state;

        self.reducer = FuncRef::new();
        self.reducer.set_instance(reducer_fn_instance);
        self.reducer.set_function(reducer_fn_name);

        self.middleware = vec![];
        self.subscriptions = vec![];
    }

    /// Returns the current state.
    #[export]
    fn state(&self, _owner: &Object) -> Dictionary<Unique> {
        self.state.duplicate()
    }

    /// Dispatches an action to update the state.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to dispatch.
    ///
    /// # Example
    ///
    /// ```
    /// const state = {
    ///     "counter": 0,
    /// }
    ///
    /// enum Action {
    ///     INCREMENT,
    ///     DECREMENT,
    /// }
    ///
    /// func reducer(state, action):
    ///     match action:
    ///         Action.INCREMENT:
    ///             return {
    ///                 "counter": state.counter + 1,
    ///             }
    ///         Action.DECREMENT:
    ///             return {
    ///                 "counter": state.counter - 1,
    ///             }
    ///
    /// func _ready():
    ///     var store = Store.new(state, self, 'reducer')
    ///     store.dispatch(Action.INCREMENT)
    /// ```
    #[export]
    fn dispatch(&mut self, _owner: &Object, action: i64) {
        if self.middleware.is_empty() {
            self.dispatch_reducer(action);
        } else {
            self.dispatch_middleware(0, action);
        }
    }

    /// Runs a single middleware function. If the middleware function returns an
    /// action then it runs the next middleware function in the middlewares array with
    /// the action returned by the previous one.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the middleware function to run from the array.
    /// * `action` - The action to pass to the middleware function.
    fn dispatch_middleware(&mut self, index: usize, action: i64) {
        if index == self.middleware.len() {
            self.dispatch_reducer(action);
            return;
        }

        let args = &[
            Variant::from_dictionary(&self.state),
            Variant::from_i64(action),
        ];
        let next = self.middleware[index].call_func(args);
        let next_to_int = next.try_to_i64();

        match next_to_int {
            Some(x) => self.dispatch_middleware(index + 1, x),
            _ => return,
        }
    }

    /// Runs the reducer for the specified action and then call any attached subscriptions.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to run the reducer for.
    fn dispatch_reducer(&mut self, action: i64) {
        let args = &[
            Variant::from_dictionary(&self.state),
            Variant::from_i64(action),
        ];
        let new_state = self.reducer.call_func(args);

        self.state = new_state.to_dictionary();

        self.dispatch_subscriptions();
    }

    /// Runs the subscriptions for the store.
    fn dispatch_subscriptions(&self) {
        let args = &[Variant::from_dictionary(&self.state)];

        for subscription in &self.subscriptions {
            subscription.call_func(args);
        }
    }

    /// Subscribes to changes to the state. When a change to the state is made, the
    /// callback function is run and passed the current state as an argument.
    ///
    /// # Arguments
    ///
    /// * `callback_fn_instance` - The instance that contains the callback function.
    /// * `param callback_fn_name` - The name of the callback function.
    ///
    /// # Example
    ///
    /// ```
    /// const state = {
    ///     "counter": 0,
    /// }
    ///
    /// enum Action {
    ///     INCREMENT,
    ///     DECREMENT,
    /// }
    ///
    /// func reducer(state, action):
    ///     match action:
    ///         Action.INCREMENT:
    ///             return {
    ///                 "counter": state.counter + 1,
    ///             }
    ///         Action.DECREMENT:
    ///             return {
    ///                 "counter": state.counter - 1,
    ///             }
    ///
    /// func _ready():
    ///     var store = Store.new()
    ///     store.set_state_and_reducer(initial_state, self, 'reducer')
    ///     store.subscribe(self, 'print_counter')
    ///
    /// func print_counter(state):
    ///     print(state.counter)
    /// ```
    #[export]
    fn subscribe(
        &mut self,
        _owner: &Object,
        subscriber_fn_instance: Ref<Object, Shared>,
        subscriber_fn_name: GodotString,
    ) {
        let subscribe_fn_ref = FuncRef::new();
        subscribe_fn_ref.set_instance(subscriber_fn_instance);
        subscribe_fn_ref.set_function(subscriber_fn_name);

        self.subscriptions.push(subscribe_fn_ref);
    }

    /// Adds a middleware function that can intercept a dispatch and modify the action
    /// to be run before it reaches the reducer.
    ///
    /// # Arguments
    ///
    /// * `middleware_fn_instance` - The instance that contains the middleware function.
    /// * `middleware_fn_name` - The name of the middleware function.
    ///
    /// # Example
    ///
    /// ```
    /// func reverse_middleware(state, action):
    ///     match action {
    ///         Action.INCREMENT:
    ///             return Action.DECREMENT
    ///         Action.DECREMENT:
    ///             return Action.INCREMENT
    ///
    /// func _ready():
    ///     var store = Store.new(state, self, 'reducer')
    ///     store.add_middleware(self, 'reverse_middleware')
    ///
    ///     # This will actually run the `DECREMENT` action because of our middleware.
    ///     store.dispatch(Action.INCREMENT)
    /// ```
    #[export]
    fn add_middleware(
        &mut self,
        _owner: &Object,
        middleware_fn_instance: Ref<Object, Shared>,
        middleware_fn_name: GodotString,
    ) {
        let middleware_fn_ref = FuncRef::new();
        middleware_fn_ref.set_instance(middleware_fn_instance);
        middleware_fn_ref.set_function(middleware_fn_name);

        self.middleware.push(middleware_fn_ref)
    }
}
