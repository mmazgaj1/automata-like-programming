use std::{marker::PhantomData, rc::Rc};

use crate::automata_state::SharedAutomataState;

pub enum NextState<Id, D> {
    Continue(SharedAutomataState<Id, D>),
    ProcessEnded,
    NotFound,
}

pub trait KeyIter<K> {
    fn next(&mut self) -> Option<K>;
}

pub struct Automata<Id, D> {
    start_state: SharedAutomataState<Id, D>,
    _data_phantom: PhantomData<D>,
}

pub enum AutomataResult<Id> {
    // Ok, // Not needed - should end because no more keys, no state could be found or state forces the end of process (no default ending).
    EmptyIter(Id),
    CouldNotFindNextState(Id),
    Error(String)
}

impl <Id, D> Automata<Id, D> {
    pub fn new<FInit: Fn() -> SharedAutomataState<Id, D>>(f_state_graph_init: FInit) -> Self {
        Self {start_state: f_state_graph_init(), _data_phantom: PhantomData{}}
    }

    pub fn run(&mut self, data: &mut D) -> AutomataResult<Id> {
        // let mut is_running = true;
        let mut current_state = Rc::clone(&self.start_state);
        // let mut current_key = Option::None;
        loop {
            let connection_execute_result = current_state.borrow().execute_next_connection(data);
            match connection_execute_result {
                Err(err_msg) => {
                    // println!("{}", err_msg); 
                    return AutomataResult::Error(err_msg);
                },
                Ok(next_state_result) => {
                    match next_state_result {
                        NextState::Continue(next_state) => current_state = next_state,
                        NextState::NotFound => return AutomataResult::CouldNotFindNextState(current_state.borrow().get_id_owned()),
                        NextState::ProcessEnded => return AutomataResult::EmptyIter(current_state.borrow().get_id_owned()),
                    };
                },
            };
        };
    }

    // pub fn data(&self) -> &D {
    //     &self.data
    // }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::{automata::AutomataResult, automata_state::{new_shared_automata_state, AutomataState, SharedAutomataState}};

    use super::{Automata, NextState};

    pub struct TestNodeHello {
        next_state: Option<SharedAutomataState<u8, String>>
    }

    impl TestNodeHello {
        pub fn new(next_state: Option<SharedAutomataState<u8, String>>) -> Self {
            Self { next_state }
        }
    }

    impl AutomataState<u8, String> for TestNodeHello {
        fn get_id_owned(&self) -> u8 {
            1
        }
        
        fn get_id(&self) -> &u8 {
            &1
        }
        
        fn execute_next_connection(&self, data: &mut String) -> Result<NextState<u8, String>, String> {
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

    impl AutomataState<u8, String> for TestNodeWorld {
        fn get_id_owned(&self) -> u8 {
            2
        }
        
        fn get_id(&self) -> &u8 {
            &2
        }
        
        fn execute_next_connection(&self, data: &mut String) -> Result<NextState<u8, String>, String> {
            data.push_str(" world");
            Result::Ok(NextState::ProcessEnded)
        }
    }

    #[test]
    fn automata_2_nodes_works() -> () {
        let mut data = String::with_capacity(11);
        // let mut key_iter = TestKeyIter::new(2, 3);
        let mut automata = Automata::new(|| {
            let world_state: SharedAutomataState<u8, String> = new_shared_automata_state(TestNodeWorld::new());
            let hello_state: SharedAutomataState<u8, String> = new_shared_automata_state(TestNodeHello::new(Option::Some(Rc::clone(&world_state))));
            hello_state
        });
        let run_res = automata.run(&mut data);
        assert!(matches!(run_res, AutomataResult::EmptyIter(2)));
        assert_eq!(data, "Hello world");
    }
}
