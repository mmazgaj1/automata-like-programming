use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{automata::NextState, automata_state::{convert_to_dyn_reference, AutomataState, SharedAutomataState}, key::AutomataKey};

pub struct SimpleStateImplementation<K: AutomataKey, D, FEntry> where FEntry: Fn(&mut D, Option<&K>) -> Result<(), String>, K: PartialEq {
    _phantom: PhantomData<D>,
    key: K,
    entry_func: FEntry,
    next_states: Vec<SharedAutomataState<K, D>>,
}

impl <K: AutomataKey, D, FEntry> SimpleStateImplementation<K, D, FEntry> where FEntry: Fn(&mut D, Option<&K>) -> Result<(), String>, K: PartialEq {
    pub fn new(key: K, entry_func: FEntry) -> Self {
        Self { key, _phantom: PhantomData{}, entry_func, next_states: Vec::new()}
    }
    pub fn register_next_state<S: 'static>(&mut self, state: &Rc<RefCell<S>>) -> () 
    where S: AutomataState<K, D>
    {
        self.next_states.push(convert_to_dyn_reference(Rc::clone(state)));
    }
}

impl<K: AutomataKey, D, FEntry> AutomataState<K, D> for SimpleStateImplementation<K, D, FEntry> where FEntry: Fn(&mut D, Option<&K>) -> Result<(), String>, K: PartialEq {
    fn get_key(&self) -> &K {
        &self.key
    }
    
    fn on_entry(&self, data: &mut D, key: Option<&K>) -> Result<(), String> {
        (self.entry_func)(data, key)
    }
    
    fn find_next_state(&self, key: &K) -> NextState<K, D> {
        for n in &self.next_states {
            if n.borrow().is_key_matching(key) {
                return NextState::Continue(Rc::clone(n));
            }
        }
        NextState::NotFound
    }
    
    fn is_key_matching(&self, key: &K) -> bool {
        &self.key == key
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::{automata::{Automata, KeyIter}, automata_state::new_shared_concrete_state, simple_impl::simple_state::SimpleStateImplementation};

    struct TestKeyIter {
        end: u8,
        current: u8,
    }

    impl TestKeyIter {
        pub fn new(start: u8, end: u8) -> Self {
            Self { end, current: start }
        }
    }

    impl KeyIter<u8> for TestKeyIter {
        fn next(&mut self) -> Option<u8> {
            if self.current >= self.end {
                return Option::None
            }
            let res = Option::Some(self.current);
            self.current += 1;
            return res;
        }
    }

    #[test]
    fn automata_with_simple_states_works() -> () {
        let data = String::with_capacity(11);
        let mut key_iter = TestKeyIter::new(2, 4);
        let mut automata = Automata::new(|| {
            let world_state = new_shared_concrete_state(SimpleStateImplementation::new(3, |d: &mut String, _| {
                d.push_str("world!");
                Result::Ok(())
            }));
            let simple_state: Rc<RefCell<SimpleStateImplementation<u8, String, _>>> = new_shared_concrete_state(SimpleStateImplementation::new(2, |d: &mut String, _| {
                d.push_str(" simple ");
                Result::Ok(())
            }));
            simple_state.borrow_mut().register_next_state(&world_state);
            let hello_state: Rc<RefCell<SimpleStateImplementation<u8, String, _>>> = new_shared_concrete_state(SimpleStateImplementation::new(1, |d: &mut String, _| {
                d.push_str("Hello");
                Result::Ok(())
            }));
            hello_state.borrow_mut().register_next_state(&simple_state);
            hello_state
        }, data);
        automata.run(&mut key_iter);
        assert_eq!(automata.data(), "Hello simple world!");
    }
}