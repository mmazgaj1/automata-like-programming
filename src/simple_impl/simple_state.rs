use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::automaton_state::{convert_to_dyn_reference, AutomatonState, SharedAutomatonState};

/// Represents data, that can provide a key which will be used while searching for next state. Usually will use iterator
/// based on a sequence.
pub trait KeyProvidingData<K> {
    fn next_key(&mut self) -> Option<K>;
}

///
/// Connection representing edge between two nodes (or one node with itself) in a graph structure. Matcher is used to
/// find the next state. Based on the key provided by the data. Each connection has a specified function which will be 
/// executed while changing to matched next state.
/// 
/// * `matcher` - Defines whether this connection should be chosen for a specified key. It's up to the user to ensure
/// that connections don't have intersecting matchers. The first connection matched for a key will always be used.
/// * `exec_function` - Operation that will be executing while changing state.
/// * `connected_state` - State that will be returned if this connection is matched. Can be the same state that this
/// connection will be assigned to.
pub struct SimpleInterStateConnection<'a, K, Id, D, E> where Id: Copy + 'a, K: 'a, D: 'a, E: 'a {
    matcher: Box<dyn Fn(&K) -> bool + 'a>,
    exec_function: Box<dyn Fn(&mut D, &K) -> Result<(), E> + 'a>,
    connected_state: SharedAutomatonState<'a, Id, D, E>,
}

impl <'a, K, Id, D, E> SimpleInterStateConnection<'a, K, Id, D, E> where Id: Copy {
    /// Creates new connection with specified matcher and a procedure that will be executed when this connection is matched.
    pub fn new<M: Fn(&K) -> bool + 'a, FExec: Fn(&mut D, &K) -> Result<(), E> + 'a, S: AutomatonState<'a, Id, D, E> + 'a>(matcher: M, exec_function: FExec, next_state: &Rc<RefCell<S>>) -> Self {
        Self { matcher: Box::new(matcher), exec_function: Box::new(exec_function), connected_state: convert_to_dyn_reference(Rc::clone(next_state)) }
    }

    /// Creates new connection with specified matcher. Does nothing when matched (designed to be used with intermediate states).
    pub fn new_no_action<M: Fn(&K) -> bool + 'a, S: AutomatonState<'a, Id, D, E> + 'a>(matcher: M, next_state: &Rc<RefCell<S>>) -> Self {
        Self::new(matcher, Self::do_nothing, next_state)
    }

    /// Does nothing
    fn do_nothing(_:&mut D, _:&K) -> Result<(), E> {
        Result::Ok(())
    }
}

/// AutomatonState implementating struct which simplifies state definition by managing list of defined connections. 
/// Depends on data for providing next key. This key is then used to match a connection from the defined list.
/// Each state has an assigned identifier which is used to inform which state did the automaton stop on.
/// Identifier is copied to the result meaning it has to implement the *Copy* trait.
pub struct SimpleStateImplementation<'a, K, Id, D, E> where D: KeyProvidingData<K>, Id: Copy{
    _phantom: PhantomData<D>,
    id: Id,
    next_states: Vec<SimpleInterStateConnection<'a, K, Id, D, E>>,
}

impl <'a, K, Id, D, E> SimpleStateImplementation<'a, K, Id, D, E> where D: KeyProvidingData<K>, Id: Copy {
    /// Creates new simple state with provided identifier.
    /// 
    /// * `id` - Identifier of this state which will be copied into result when automaton stops on this state.
    pub fn new(id: Id) -> Self {
        Self { _phantom: PhantomData{}, next_states: Vec::new(), id}
    }

    /// Adds connection to possible next states of current state.
    pub fn register_connection(&mut self, connection: SimpleInterStateConnection<'a, K, Id, D, E>) -> () 
    {
        self.next_states.push(connection);
    }
}

impl<'a, K, Id, D, E> AutomatonState<'a, Id, D, E> for SimpleStateImplementation<'a, K, Id, D, E> where D: KeyProvidingData<K>, Id: Copy {
    /// Returns owned copy of identifier of this state.
    fn get_id_owned(&self) -> Id {
        self.id
    }

    /// Returns identifier of this state.
    fn get_id(&self) -> &Id {
        &self.id
    }

    /// Finds connection by popping key from key iterator. Executes assigned function and returns next state if everything goes
    /// alright. 
    fn execute_next_connection(&self, data: &mut D) -> Result<crate::automaton::NextState<'a, Id, D, E>, E> {
        let next_key = data.next_key();
        if let Option::Some(k) = next_key {
            for c in &self.next_states {
                if (c.matcher)(&k) {
                    (c.exec_function)(data, &k)?;
                    return Result::Ok(crate::automaton::NextState::Continue(Rc::clone(&c.connected_state)));
                }
            }
            Result::Ok(crate::automaton::NextState::NotFound)
        } else {
            Result::Ok(crate::automaton::NextState::ProcessEnded)
        }
    }
}

#[cfg(test)]
mod test {
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

    mod automaton_test {
        use crate::{automaton::{Automaton, AutomatonResult}, automaton_state::new_shared_concrete_state, simple_impl::simple_state::{test::TestData, SimpleInterStateConnection, SimpleStateImplementation}};

        #[test]
        fn automaton_with_simple_states_works() -> () {
            let mut data = TestData::new(1, 4);
            let mut automaton = Automaton::new(|| {
                let world_state = new_shared_concrete_state(SimpleStateImplementation::new(3));
                let simple_state = new_shared_concrete_state(SimpleStateImplementation::new(2));
                simple_state.borrow_mut().register_connection(SimpleInterStateConnection::new(|k| k == &2, |d: &mut TestData, _| {
                    d.append_text(" simple ");
                    let res: Result<(), String> = Result::Ok(());
                    res
                }, &world_state));
                let hello_state = new_shared_concrete_state(SimpleStateImplementation::new(1));
                hello_state.borrow_mut().register_connection(SimpleInterStateConnection::new(|k| k == &1, |d: &mut TestData, _| {
                    d.append_text("Hello");
                    Result::Ok(())
                }, &simple_state));
                world_state.borrow_mut().register_connection(SimpleInterStateConnection::new(|k| k == &3, |d: &mut TestData, _| {
                    d.append_text("world!");
                    Result::Ok(())
                }, &hello_state));
                hello_state
            });
            let run_result = automaton.run(&mut data);
            assert_eq!(data.data(), "Hello simple world!");
            assert!(matches!(run_result, AutomatonResult::EmptyIter(1)));
        }

        // TBF I don't know if this situation should be Ok or Err
        #[test]
        fn automaton_with_simple_states_works_no_next_state_found() -> () {
            let mut data = TestData::new(2, 3);
            let mut automaton = Automaton::new(|| {
                new_shared_concrete_state(SimpleStateImplementation::new(1))
            });
            let run_result: AutomatonResult<u32, String> = automaton.run(&mut data);
            assert_eq!(data.data(), "");
            assert!(matches!(run_result, AutomatonResult::CouldNotFindNextState(1)));
        }
    }
}
