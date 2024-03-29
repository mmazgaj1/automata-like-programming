use crate::{automata::NextState, key::AutomataKey};

pub trait AutomataState<K: AutomataKey, D> {
    fn get_key(&self) -> &K;
    fn on_entry(&self, data: &mut D) -> Result<(), String>;
    // fn on_exit(&self, data: &mut D) -> Result<(), String>;
    fn find_next_state(&self, key: &K) -> NextState<K, D>;
}