use std::{cell::RefCell, rc::Rc};

use crate::automata::NextState;

pub trait AutomataState<'a, Id, D> {
    /// Identifier used for identifying current state. Has to return owned data, because it will be used in returned value.
    fn get_id_owned(&self) -> Id;
    // TODO: Dunno, maybe should just use the owned id everywhere
    fn get_id(&self) -> &Id;
    /// Based on graph like structure of automata representing execution of an action while going along an edge between states.
    /// Usually state will choose next state based on its inner state and execute action assigned to it (or do nothing if no more states can be found).
    /// Implementations have to rely on own mechanism for determining execution sequence.
    fn execute_next_connection(&self, data: &mut D) -> Result<NextState<'a, Id, D>, String>;
}

pub type SharedAutomataState<'a, Id, D> = Rc<RefCell<dyn AutomataState<'a, Id, D> + 'a>>;

/// Creates shared reference for given state. Returned type signature is: Rc<RefCell<dyn AutomataState>>
pub fn new_shared_automata_state<'a, Id, D, S: AutomataState<'a, Id, D> + 'a>(state: S) -> SharedAutomataState<'a, Id, D> {
    Rc::new(RefCell::new(state))
}

/// Creates shared reference for given state. Returned type signature is: Rc<RefCell<S>> where S is a concrete
/// implementation of AutomataState.
pub fn new_shared_concrete_state<'a, Id, D, S: AutomataState<'a, Id, D> + 'a>(state: S) -> Rc<RefCell<S>> {
    Rc::new(RefCell::new(state))
}

/// Converts type signature from using concrete implementation type to 'dyn AutomataState'.
pub fn convert_to_dyn_reference<'a, Id, D, S: AutomataState<'a, Id, D> + 'a>(state: Rc<RefCell<S>>) -> SharedAutomataState<'a, Id, D> {
    state as SharedAutomataState<'a, Id, D>
}
