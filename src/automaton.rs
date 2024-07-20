use std::{marker::PhantomData, rc::Rc};

use crate::automaton_state::SharedAutomatonState;

/// Result of an attempt of determining next target state.
pub enum NextState<'a, Id, D> {
    /// Automaton should take provided state for the next iteration.
    Continue(SharedAutomatonState<'a, Id, D>),
    /// The input data has ended so there is no way for matching next state.
    ProcessEnded,
    /// There are no possible target states for received input data.
    NotFound,
}

/// Iterator for providing next key.
pub trait KeyIter<K> {
    fn next(&mut self) -> Option<K>;
}

/// Finite-state automaton that crawls around a specified graph until no more state changes can be done.
pub struct Automaton<'a, Id, D> {
    start_state: SharedAutomatonState<'a, Id, D>,
    _data_phantom: PhantomData<D>,
}

/// Provides information on why automaton has stopped executing.
pub enum AutomatonResult<Id> {
    // Ok, // Not needed - should end because no more keys, no state could be found or state forces the end of process (no default ending).
    /// Automaton execution ended because no more keys could be extracted. Contains identifier of current state in automaton execution - no more
    /// keys could be extracted after reaching this state.
    EmptyIter(
        Id
    ),
    /// No connection could be matched for a key. Contains identifier of current state in automaton execution - no connections could be found on this state for given key.
    CouldNotFindNextState(
        Id
    ),
    /// An error occured while executing function assigned to connection. Contains error message from execution
    Error(
        String
    )
}

impl <'a, Id, D> Automaton<'a, Id, D> {
    /// Creates new automaton with graph initiated by specified function.
    pub fn new<FInit: Fn() -> SharedAutomatonState<'a, Id, D>>(f_state_graph_init: FInit) -> Self {
        Self {start_state: f_state_graph_init(), _data_phantom: PhantomData{}}
    }

    /// Starts automaton with given data.
    pub fn run(&mut self, data: &mut D) -> AutomatonResult<Id> {
        let mut current_state = Rc::clone(&self.start_state);
        loop {
            let connection_execute_result = current_state.borrow().execute_next_connection(data);
            match connection_execute_result {
                Err(err_msg) => {
                    return AutomatonResult::Error(err_msg);
                },
                Ok(next_state_result) => {
                    match next_state_result {
                        NextState::Continue(next_state) => current_state = next_state,
                        NextState::NotFound => return AutomatonResult::CouldNotFindNextState(current_state.borrow().get_id_owned()),
                        NextState::ProcessEnded => return AutomatonResult::EmptyIter(current_state.borrow().get_id_owned()),
                    };
                },
            };
        };
    }
}

#[cfg(test)]
pub mod test {
    use std::rc::Rc;

    use crate::{automaton::AutomatonResult, automaton_state::{new_shared_automaton_state, AutomatonState, SharedAutomatonState}};

    use super::{Automaton, NextState};

    pub struct TestNodeHello<'a> {
        next_state: Option<SharedAutomatonState<'a, u8, String>>
    }

    impl<'a> TestNodeHello <'a> {
        pub fn new(next_state: Option<SharedAutomatonState<'a, u8, String>>) -> Self {
            Self { next_state }
        }
    }

    impl <'a> AutomatonState<'a, u8, String> for TestNodeHello<'a> {
        fn get_id_owned(&self) -> u8 {
            1
        }
        
        fn get_id(&self) -> &u8 {
            &1
        }
        
        fn execute_next_connection(&self, data: &mut String) -> Result<NextState<'a, u8, String>, String> {
            data.push_str("Hello");
            if let Option::Some(nxt_state) = &self.next_state {
                Result::Ok(NextState::Continue(Rc::clone(nxt_state)))
            } else {
                Result::Ok(NextState::NotFound)
            }
        }
    }

    pub struct TestNodeWorld {
    }

    impl TestNodeWorld {
        pub fn new() -> Self {
            Self {  }
        }
    }

    impl <'a> AutomatonState<'a, u8, String> for TestNodeWorld {
        fn get_id_owned(&self) -> u8 {
            2
        }
        
        fn get_id(&self) -> &u8 {
            &2
        }
        
        fn execute_next_connection(&self, data: &mut String) -> Result<NextState<'a, u8, String>, String> {
            data.push_str(" world");
            Result::Ok(NextState::ProcessEnded)
        }
    }

    #[test]
    fn automaton_2_nodes_works() -> () {
        let mut data = String::with_capacity(11);
        let mut automaton = Automaton::new(|| {
            let world_state: SharedAutomatonState<u8, String> = new_shared_automaton_state(TestNodeWorld::new());
            let hello_state: SharedAutomatonState<u8, String> = new_shared_automaton_state(TestNodeHello::new(Option::Some(Rc::clone(&world_state))));
            hello_state
        });
        let run_res = automaton.run(&mut data);
        assert!(matches!(run_res, AutomatonResult::EmptyIter(2)));
        assert_eq!(data, "Hello world");
    }
}
