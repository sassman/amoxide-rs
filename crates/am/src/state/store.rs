use super::{actions::Action, middleware::MiddlewareFn, reducers::Reducer, state::ReduxState};

/// The app store, generic over the app state `S`
pub struct Store<State> {
    state: State,
    reducer: Reducer<State>,
    middlewares: Vec<MiddlewareFn<State>>,
}

impl<State: ReduxState> Store<State> {
    pub fn new(
        state: State,
        reducer: Reducer<State>,
        middlewares: Vec<MiddlewareFn<State>>,
    ) -> Self {
        Self {
            state,
            reducer,
            middlewares,
        }
    }

    pub fn dispatch(&mut self, action: Action) -> crate::Result<()> {
        let state = &self.state;

        let mut new_action = Some(action);
        for middleware in self.middlewares.iter() {
            if let Some(action) = new_action {
                new_action = middleware(state, &action);
            }
        }

        if let Some(action) = new_action {
            let state = (self.reducer)(state, &action)?;
            self.state = state;
        }
        Ok(())
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{middleware::*, reducers::app_reducer, state::AppState};

    use super::*;

    #[test]
    fn test_name() {
        let state = AppState::default();
        let middlewares = vec![logging_middleware::<AppState>()];
        let reducer = app_reducer;

        let mut store = Store::new(state, reducer, middlewares);

        store
            .dispatch(Action::SetEnv("default".to_string()))
            .unwrap();

        // assert_eq!(store.get_state(), "default");
    }
}
