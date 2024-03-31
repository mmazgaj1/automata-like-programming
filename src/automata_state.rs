use std::{cell::RefCell, rc::Rc};

use crate::{automata::NextState, key::AutomataKey};

pub trait AutomataState<K: AutomataKey, D> {
    /// Identifier used for finding next state. Has to be unique within all states assigned as possible continuation for current state.
    fn get_key(&self) -> &K;
    ///
    /// Called after the state is chosen to be executed.
    /// 
    /// * `data` - Value passed to all executed states. Always mutable reference to the same data.
    fn on_entry(&self, data: &mut D, key: Option<&K>) -> Result<(), String>;
    /// Searches for state identified by key from all assigned possible continuation states.
    fn find_next_state(&self, key: &K) -> NextState<K, D>;
    fn is_key_matching(&self, key: &K) -> bool;
}

pub type SharedAutomataState<K, D> = Rc<RefCell<dyn AutomataState<K, D>>>;

pub fn new_shared_automata_state<K: AutomataKey, D, S: AutomataState<K, D> + 'static>(state: S) -> SharedAutomataState<K, D> {
    Rc::new(RefCell::new(state))
}

pub fn new_shared_concrete_state<K: AutomataKey, D, S: AutomataState<K, D> + 'static>(state: S) -> Rc<RefCell<S>> {
    Rc::new(RefCell::new(state))
}

pub fn convert_to_dyn_reference<K: AutomataKey, D, S: AutomataState<K, D> + 'static>(state: Rc<RefCell<S>>) -> SharedAutomataState<K, D> {
    state as SharedAutomataState<K, D>
}