use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{automata::NextState, automata_state::{convert_to_dyn_reference, AutomataState, SharedAutomataState}};

pub struct SimpleStateImplementation<K, Id, D, FEntry, FExit> where FEntry: Fn(&mut D, Option<&K>) -> Result<(), String>, K: PartialEq, FExit: Fn(&mut D, Option<&Id>) -> Result<(), String> {
    _phantom: PhantomData<D>,
    key: K,
    id: Id,
    entry_func: FEntry,
    exit_func: FExit,
    next_states: Vec<SharedAutomataState<K, Id, D>>,
}

impl <K, Id, D, FEntry, FExit> SimpleStateImplementation<K, Id, D, FEntry, FExit> where FEntry: Fn(&mut D, Option<&K>) -> Result<(), String>, K: PartialEq, FExit: Fn(&mut D, Option<&Id>) -> Result<(), String> {
    pub fn new(key: K, entry_func: FEntry, id: Id, exit_func: FExit) -> Self {
        Self { key, _phantom: PhantomData{}, entry_func, next_states: Vec::new(), id, exit_func}
    }
    pub fn register_next_state<S: 'static>(&mut self, state: &Rc<RefCell<S>>) -> () 
    where S: AutomataState<K, Id, D>
    {
        self.next_states.push(convert_to_dyn_reference(Rc::clone(state)));
    }
}

impl<K, Id, D, FEntry, FExit> AutomataState<K, Id, D> for SimpleStateImplementation<K, Id, D, FEntry, FExit> where FEntry: Fn(&mut D, Option<&K>) -> Result<(), String>, K: PartialEq, Id: Copy, FExit: Fn(&mut D, Option<&Id>) -> Result<(), String> {
    fn get_id_owned(&self) -> Id {
        self.id
    }
    
    fn on_entry(&self, data: &mut D, key: Option<&K>) -> Result<(), String> {
        (self.entry_func)(data, key)
    }
    
    fn find_next_state(&self, key: &K) -> NextState<K, Id, D> {
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
    
    fn on_exit(&self, data: &mut D, next_state_id: Option<&Id>) -> Result<(), String> {
        (self.exit_func)(data, next_state_id)
    }
    
    fn get_id(&self) -> &Id {
        &self.id
    }
}

pub fn empty_exit_func<D, Id>(_: &mut D, _: Option<&Id>) -> Result<(), String> {
    Result::Ok(())
}

#[cfg(test)]
mod test {
    use crate::{automata::{Automata, AutomataResult, KeyIter}, automata_state::new_shared_concrete_state, simple_impl::simple_state::{empty_exit_func, SimpleStateImplementation}};

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
            }, 3, empty_exit_func));
            let simple_state = new_shared_concrete_state(SimpleStateImplementation::new(2, |d: &mut String, _| {
                d.push_str(" simple ");
                Result::Ok(())
            }, 2, empty_exit_func));
            simple_state.borrow_mut().register_next_state(&world_state);
            let hello_state = new_shared_concrete_state(SimpleStateImplementation::new(1, |d: &mut String, _| {
                d.push_str("Hello");
                Result::Ok(())
            }, 1, empty_exit_func));
            hello_state.borrow_mut().register_next_state(&simple_state);
            hello_state
        }, data);
        assert!(matches!(automata.run(&mut key_iter), AutomataResult::EmptyIter(3)));
        assert_eq!(automata.data(), "Hello simple world!");
    }
}