use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::automata_state::{convert_to_dyn_reference, AutomataState, SharedAutomataState};

/// Represents data, that can provide a key which will be used while searching for next state. Usually will use iterator
/// based on a sequence.
pub trait KeyProvidingData<K> {
    fn next_key(&mut self) -> Option<K>;
}

pub struct SimpleInterStateConnection<K, Id, D> where Id: Copy{
    matcher: Box<dyn Fn(&K) -> bool>,
    exec_function: Box<dyn Fn(&mut D) -> Result<(), String>>,
    connected_state: SharedAutomataState<Id, D>,
}

impl <K, Id, D> SimpleInterStateConnection<K, Id, D> where Id: Copy {
    pub fn new<M: Fn(&K) -> bool + 'static, FExec: Fn(&mut D) -> Result<(), String> + 'static, S: AutomataState<Id, D> + 'static>(matcher: M, exec_function: FExec, next_state: &Rc<RefCell<S>>) -> Self {
        Self { matcher: Box::new(matcher), exec_function: Box::new(exec_function), connected_state: convert_to_dyn_reference(Rc::clone(next_state)) }
    }
}

pub struct SimpleStateImplementation<K, Id, D> where D: KeyProvidingData<K>, Id: Copy{
    _phantom: PhantomData<D>,
    id: Id,
    next_states: Vec<SimpleInterStateConnection<K, Id, D>>,
}

impl <K, Id, D> SimpleStateImplementation<K, Id, D> where D: KeyProvidingData<K>, Id: Copy {
    pub fn new(id: Id) -> Self {
        Self { _phantom: PhantomData{}, next_states: Vec::new(), id}
    }

    pub fn register_connection(&mut self, connection: SimpleInterStateConnection<K, Id, D>) -> () 
    {
        self.next_states.push(connection);
    }

    pub fn register_next_state<M: Fn(&K) -> bool + 'static, FExec: Fn(&mut D) -> Result<(), String> + 'static, S: AutomataState<Id, D> + 'static>(&mut self, matcher: M, exec_function: FExec, state: &Rc<RefCell<S>>) -> () 
    {
        self.register_connection(SimpleInterStateConnection::new(matcher, exec_function, state));
    }
}

impl<K, Id, D> AutomataState<Id, D> for SimpleStateImplementation<K, Id, D> where D: KeyProvidingData<K>, Id: Copy {
    fn get_id_owned(&self) -> Id {
        self.id
    }
    
    fn get_id(&self) -> &Id {
        &self.id
    }
    
    fn execute_next_connection(&self, data: &mut D) -> Result<crate::automata::NextState<Id, D>, String> {
        let next_key = data.next_key();
        if let Option::Some(k) = next_key {
            for c in &self.next_states {
                if (c.matcher)(&k) {
                    (c.exec_function)(data)?;
                    return Result::Ok(crate::automata::NextState::Continue(Rc::clone(&c.connected_state)));
                }
            }
            Result::Ok(crate::automata::NextState::NotFound)
        } else {
            Result::Ok(crate::automata::NextState::ProcessEnded)
        }
    }
}

pub fn empty_exit_func<D, Id>(_: &mut D, _: Option<&Id>) -> Result<(), String> {
    Result::Ok(())
}

#[cfg(test)]
mod test {
    use crate::{automata::{Automata, AutomataResult}, automata_state::new_shared_concrete_state, simple_impl::simple_state::{SimpleInterStateConnection, SimpleStateImplementation}};

    use super::KeyProvidingData;

    struct TestData {
        buffer: String,
        end: u8,
        current: u8,
    }

    impl TestData {
        pub fn new(start: u8, end: u8) -> Self {
            Self { buffer: String::new(), end, current: start }
        }

        pub fn append_text(&mut self, text: &str) -> () {
            self.buffer.push_str(text);
        }

        pub fn data(&self) -> &String {
            &self.buffer
        }
    }

    impl KeyProvidingData<u8> for TestData {
        fn next_key(&mut self) -> Option<u8> {
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
        let data = TestData::new(1, 4);
        let mut automata = Automata::new(|| {
            let world_state = new_shared_concrete_state(SimpleStateImplementation::new(3));
            let simple_state = new_shared_concrete_state(SimpleStateImplementation::new(2));
            simple_state.borrow_mut().register_next_state(|k| k == &2, |d: &mut TestData| {
                d.append_text(" simple ");
                Result::Ok(())
            }, &world_state);
            let hello_state = new_shared_concrete_state(SimpleStateImplementation::new(1));
            hello_state.borrow_mut().register_next_state(|k| k == &1, |d: &mut TestData| {
                d.append_text("Hello");
                Result::Ok(())
            }, &simple_state);
            world_state.borrow_mut().register_connection(SimpleInterStateConnection::new(|k| k == &3, |d: &mut TestData| {
                d.append_text("world!");
                Result::Ok(())
            }, &hello_state));
            hello_state
        }, data);
        let run_result = automata.run();
        assert_eq!(automata.data().data(), "Hello simple world!");
        assert!(matches!(run_result, AutomataResult::EmptyIter(1)));
    }

    // TBF I don't know if this situation should be Ok or Err
    #[test]
    fn automata_with_simple_states_works_no_next_state_found() -> () {
        let data = TestData::new(2, 3);
        let mut automata = Automata::new(|| {
            new_shared_concrete_state(SimpleStateImplementation::new(1))
        }, data);
        let run_result = automata.run();
        assert_eq!(automata.data().data(), "");
        assert!(matches!(run_result, AutomataResult::CouldNotFindNextState(1)));
    }
}
