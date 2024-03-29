use std::{cell::RefCell, rc::Rc};

use crate::{automata_state::AutomataState, key::AutomataKey};

pub enum NextState<K: AutomataKey, D> {
    Continue(Rc<RefCell<Box<dyn AutomataState<K, D>>>>),
    NotFound,
}

pub trait KeyIter<K: AutomataKey> {
    fn next(&mut self) -> Option<K>;
}

pub struct Automata<K: AutomataKey, D, KIter: KeyIter<K>> {
    start_state: Rc<RefCell<Box<dyn AutomataState<K, D>>>>,
    data: D,
    key_iter: KIter,
}

impl <K: AutomataKey, D, KIter: KeyIter<K>> Automata<K, D, KIter> {
    pub fn new<FInit: Fn() -> Rc<RefCell<Box<dyn AutomataState<K, D>>>>>(f_state_graph_init: FInit, data: D, key_iter: KIter) -> Self {
        Self {start_state: f_state_graph_init(), data, key_iter}
    }

    pub fn run(&mut self) -> () {
        let mut is_running = true;
        let mut current_state = Rc::clone(&self.start_state);
        while is_running {
            match current_state.borrow().on_entry(&mut self.data) {
                Err(err_msg) => {
                    println!("{}", err_msg); 
                    return;
                },
                _ => (),
            }
            if let Option::Some(next_key) = self.key_iter.next() {
                let next_state_find = current_state.borrow().find_next_state(&next_key);
                if let NextState::Continue(next_state) = next_state_find {
                    current_state = next_state;
                } else {
                    is_running = false;
                }
            } else {
                is_running = false;
            }

        }
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::{automata_state::AutomataState, key::AutomataKey};

    use super::{Automata, KeyIter, NextState};

    // struct TestSequenceAutomataKey {
    //     value: u8,
    // }

    impl AutomataKey for u8 {

    }

    pub struct TestNodeHello {
        next_state: Option<Rc<RefCell<Box<dyn AutomataState<u8, String>>>>>
    }

    impl TestNodeHello {
        pub fn new(next_state: Option<Rc<RefCell<Box<dyn AutomataState<u8, String>>>>>) -> Self {
            Self { next_state }
        }
    }

    impl AutomataState<u8, String> for TestNodeHello {
        fn get_key(&self) -> &u8 {
            &1
        }
    
        fn on_entry(&self, data: &mut String) -> Result<(), String> {
            data.push_str("Hello");
            Result::Ok(())
        }
    
        fn find_next_state(&self, key: &u8) -> super::NextState<u8, String> {
            if key == &2 {
                if let Option::Some(world_state) = &self.next_state {
                    return NextState::Continue(Rc::clone(world_state))
                }
            }
            super::NextState::NotFound
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
        fn get_key(&self) -> &u8 {
            &2
        }
    
        fn on_entry(&self, data: &mut String) -> Result<(), String> {
            data.push_str(" world");
            Result::Ok(())
        }
    
        fn find_next_state(&self, _: &u8) -> super::NextState<u8, String> {
            super::NextState::NotFound
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
        let mut automata = Automata::new(|| {
            let world_state: Rc<RefCell<Box<dyn AutomataState<u8, String>>>> = Rc::new(RefCell::new(Box::new(TestNodeWorld::new())));
            let hello_state: Rc<RefCell<Box<dyn AutomataState<u8, String>>>> = Rc::new(RefCell::new(Box::new(TestNodeHello::new(Option::Some(Rc::clone(&world_state))))));
            hello_state
        }, data, TestKeyIter::new(2, 3));
        automata.run();
        assert_eq!(automata.data, "Hello world");
    }
}