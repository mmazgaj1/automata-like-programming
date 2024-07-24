use std::{cell::RefCell, rc::Rc};

use crate::automaton::NextState;

/// Representation of a node in automaton graph. States act as stop points for an automaton where next states are determined or for
/// halting the execution when no more state changes can be done.
pub trait AutomatonState<'a, Id, D, E> {
    /// Owned identifier used for identifying current state.
    fn get_id_owned(&self) -> Id;
    /// Reference to identifier user for identifying current state.
    fn get_id(&self) -> &Id;

    /// Represents change of current state in graph. Provides state to be executed by automaton. Implementations should use this method for executing operations connected with
    /// state change.
    fn execute_next_connection(&self, data: &mut D) -> Result<NextState<'a, Id, D, E>, E>;
}

pub type SharedAutomatonState<'a, Id, D, E> = Rc<RefCell<dyn AutomatonState<'a, Id, D, E> + 'a>>;

/// Creates shared reference for given state. Returned type signature is: `Rc<RefCell<dyn AutomatonState>>`
pub fn new_shared_automaton_state<'a, Id, D, E, S: AutomatonState<'a, Id, D, E> + 'a>(state: S) -> SharedAutomatonState<'a, Id, D, E> {
    Rc::new(RefCell::new(state))
}

/// Creates shared reference for given state. Returned type signature is: `Rc<RefCell<S>>` where S is a concrete
/// implementation of AutomatonState.
pub fn new_shared_concrete_state<'a, Id, D, E, S: AutomatonState<'a, Id, D, E> + 'a>(state: S) -> Rc<RefCell<S>> {
    Rc::new(RefCell::new(state))
}

/// Converts type signature from using concrete implementation type to `dyn AutomatonState`.
pub fn convert_to_dyn_reference<'a, Id, D, E, S: AutomatonState<'a, Id, D, E> + 'a>(state: Rc<RefCell<S>>) -> SharedAutomatonState<'a, Id, D, E> {
    state as SharedAutomatonState<'a, Id, D, E>
}
