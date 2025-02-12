use super::{actions::Action, state::ReduxState};

/// A middleware function, is pure function that takes
/// the current state and an action it returns a new action or None.
///
/// The new action will be dispatched to the store, instead of the original action.
/// None will be ignored, and leads to the original action being dispatched.
///
/// A chain of middleware functions is usually processed, so that the returning action,
/// will be passed to the next middleware function.
pub type MiddlewareFn<State> = fn(&State, &Action) -> Option<Action>;

pub fn logging_middleware<State: ReduxState>() -> MiddlewareFn<State> {
    |state, action| {
        println!(
            "[*] logMiddleware: Action: {:?}, State: {:?}",
            action, state
        );
        None
    }
}
