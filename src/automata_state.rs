use std::{cell::RefCell, rc::Rc};

use crate::automata::NextState;

pub trait AutomataState<K, Id, D> {
    /// Identifier used for finding next state. Has to be unique within all states assigned as possible continuation for current state.
    fn get_id_owned(&self) -> Id;
    fn get_id(&self) -> &Id;
    ///
    /// Called after the state is chosen to be executed.
    /// 
    /// * `data` - Value passed to all executed states. Always mutable reference to the same data.
    fn on_entry(&self, data: &mut D, key: Option<&K>) -> Result<(), String>;
    /// Called when next state is known or automata ends it's life.
    fn on_exit(&self, data: &mut D, next_state_id: Option<&Id>) -> Result<(), String>;
    /// Searches for state identified by key from all assigned possible continuation states.
    fn find_next_state(&self, key: &K) -> NextState<K, Id, D>;
    fn is_key_matching(&self, key: &K) -> bool;
}

pub type SharedAutomataState<K, Id, D> = Rc<RefCell<dyn AutomataState<K, Id, D>>>;

pub fn new_shared_automata_state<K, Id, D, S: AutomataState<K, Id, D> + 'static>(state: S) -> SharedAutomataState<K, Id, D> {
    Rc::new(RefCell::new(state))
}

pub fn new_shared_concrete_state<K, Id, D, S: AutomataState<K, Id, D> + 'static>(state: S) -> Rc<RefCell<S>> {
    Rc::new(RefCell::new(state))
}

pub fn convert_to_dyn_reference<K, Id, D, S: AutomataState<K, Id, D> + 'static>(state: Rc<RefCell<S>>) -> SharedAutomataState<K, Id, D> {
    state as SharedAutomataState<K, Id, D>
}