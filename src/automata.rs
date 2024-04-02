use std::rc::Rc;

use crate::automata_state::SharedAutomataState;

pub enum NextState<K, Id, D> {
    Continue(SharedAutomataState<K, Id, D>),
    NotFound,
}

pub trait KeyIter<K> {
    fn next(&mut self) -> Option<K>;
}

pub struct Automata<K, Id, D> {
    start_state: SharedAutomataState<K, Id, D>,
    data: D,
    // key_iter: KIter,
}

// #[derive(Debug, PartialEq)]
pub enum AutomataResult<K, Id> {
    // Ok, // Not necessary - should end because no more keys, no state could be found or state forces the end of process (no default ending).
    EmptyIter(Id),
    CouldNotFindNextState(K, Id),
    Error(String)
}

impl <K, Id, D> Automata<K, Id, D> {
    pub fn new<FInit: Fn() -> SharedAutomataState<K, Id, D>>(f_state_graph_init: FInit, data: D) -> Self {
        Self {start_state: f_state_graph_init(), data}
    }

    pub fn run<KIter: KeyIter<K>>(&mut self, key_iter: &mut KIter) -> AutomataResult<K, Id> {
        // let mut is_running = true;
        let mut current_state = Rc::clone(&self.start_state);
        let mut current_key = Option::None;
        loop {
            match current_state.borrow().on_entry(&mut self.data, current_key.as_ref()) {
                Err(err_msg) => {
                    // println!("{}", err_msg); 
                    return AutomataResult::Error(err_msg);
                },
                _ => (),
            }
            current_key = key_iter.next();
            if let Option::Some(next_key) = &current_key {
                let next_state_find = current_state.borrow().find_next_state(&next_key);
                if let NextState::Continue(next_state) = next_state_find {
                    if let Result::Err(err_msg) = current_state.borrow().on_exit(&mut self.data, Option::Some(next_state.borrow().get_id())) {
                        return AutomataResult::Error(err_msg)
                    }
                    current_state = next_state;
                } else {
                    return if let Result::Err(err_msg) = current_state.borrow().on_exit(&mut self.data, Option::None) {
                        AutomataResult::Error(err_msg)
                    } else {
                        AutomataResult::CouldNotFindNextState(current_key.unwrap(), current_state.borrow().get_id_owned())
                    };
                }
            } else {
                return AutomataResult::EmptyIter(current_state.borrow().get_id_owned())
            }
        }
    }

    pub fn data(&self) -> &D {
        &self.data
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::{automata::AutomataResult, automata_state::{new_shared_automata_state, AutomataState, SharedAutomataState}};

    use super::{Automata, KeyIter, NextState};

    pub struct TestNodeHello {
        next_state: Option<SharedAutomataState<u8, u8, String>>
    }

    impl TestNodeHello {
        pub fn new(next_state: Option<SharedAutomataState<u8, u8, String>>) -> Self {
            Self { next_state }
        }
    }

    impl AutomataState<u8, u8, String> for TestNodeHello {
        fn get_id_owned(&self) -> u8 {
            1
        }
    
        fn on_entry(&self, data: &mut String, _: Option<&u8>) -> Result<(), String> {
            data.push_str("Hello");
            Result::Ok(())
        }
    
        fn find_next_state(&self, key: &u8) -> super::NextState<u8, u8, String> {
            if &2 == key {
                if let Option::Some(world_state) = &self.next_state {
                    return NextState::Continue(Rc::clone(world_state))
                }
            }
            super::NextState::NotFound
        }
        
        fn is_key_matching(&self, key: &u8) -> bool {
            &1 == key
        }
        
        fn on_exit(&self, _: &mut String, _: Option<&u8>) -> Result<(), String> {
            Result::Ok(())
        }
        
        fn get_id(&self) -> &u8 {
            &1
        }
    }

    pub struct TestNodeWorld {
    }

    impl TestNodeWorld {
        pub fn new() -> Self {
            Self {  }
        }
    }

    impl AutomataState<u8, u8, String> for TestNodeWorld {
        fn get_id_owned(&self) -> u8 {
            2
        }
    
        fn on_entry(&self, data: &mut String, _: Option<&u8>) -> Result<(), String> {
            data.push_str(" world");
            Result::Ok(())
        }
    
        fn find_next_state(&self, _: &u8) -> super::NextState<u8, u8, String> {
            super::NextState::NotFound
        }
        
        fn is_key_matching(&self, key: &u8) -> bool {
            &2 == key
        }
        
        fn on_exit(&self, _: &mut String, _: Option<&u8>) -> Result<(), String> {
            Result::Ok(())
        }
        
        fn get_id(&self) -> &u8 {
            &2
        }
    }

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
    fn automata_2_nodes_works() -> () {
        let data = String::with_capacity(11);
        let mut key_iter = TestKeyIter::new(2, 3);
        let mut automata = Automata::new(|| {
            let world_state: SharedAutomataState<u8, u8, String> = new_shared_automata_state(TestNodeWorld::new());
            let hello_state: SharedAutomataState<u8, u8, String> = new_shared_automata_state(TestNodeHello::new(Option::Some(Rc::clone(&world_state))));
            hello_state
        }, data);
        let run_res = automata.run(&mut key_iter);
        assert!(matches!(run_res, AutomataResult::EmptyIter(2)));
        assert_eq!(automata.data, "Hello world");
    }
}