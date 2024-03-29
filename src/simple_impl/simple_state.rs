use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{automata::NextState, automata_state::AutomataState, key::AutomataKey};

pub struct SimpleStateImplementation<K: AutomataKey, D, FEntry> where FEntry: Fn(&mut D) -> Result<(), String> {
    _phantom: PhantomData<D>,
    key: K,
    entry_func: FEntry,
    next_states: Vec<Rc<RefCell<Box<dyn AutomataState<K, D>>>>>,
}

impl <K: AutomataKey, D, FEntry> SimpleStateImplementation<K, D, FEntry> where FEntry: Fn(&mut D) -> Result<(), String> {
    pub fn new(key: K, entry_func: FEntry) -> Self {
        Self { key, _phantom: PhantomData{}, entry_func, next_states: Vec::new()}
    }
}

impl<K: AutomataKey, D, FEntry> AutomataState<K, D> for SimpleStateImplementation<K, D, FEntry> where FEntry: Fn(&mut D) -> Result<(), String> {
    fn get_key(&self) -> &K {
        &self.key
    }
    
    fn on_entry(&self, data: &mut D) -> Result<(), String> {
        (self.entry_func)(data)
    }
    
    // fn on_exit(&self, _: &mut D) -> Result<(), String> {
    //     Result::Ok(())
    // }
    
    fn find_next_state(&self, key: &K) -> NextState<K, D> {
        for n in &self.next_states {
            if n.borrow().get_key() == key {
                return NextState::Continue(Rc::clone(n));
            }
        }
        NextState::NotFound
    }
}
