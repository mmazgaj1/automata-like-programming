[![crates.io](https://img.shields.io/crates/v/automata_like_programming.svg)](https://crates.io/crates/automata-like-programming)

# Automata like programming

Automaton is a machine that automatically follows a sequence of operations. It can be described by a state diagram where states are connected through edges marked with an input symbol that will make the automaton jump to the connected state. This library provides a simple workflow for implementing such automata by either using simple implementation (module simple_impl) or by manually implementing states. Automaton stops executing when no more connections are available or if the input ends. During the whole run automaton uses the same mutable reference to data and can execute custom operation on interstate transitions (operation depends on state implementation). Automaton can exit with an error if it happens during state jump.

## Example automaton that concatenates predefined strings into "FooBar"

This automaton has predefined sequence of execution (there is no input that would control order for executed states). This means that it will run until the last state will be achieved (sequential execution).

```rust
use std::rc::Rc;
use automata_like_programming::{
        automaton::AutomatonResult, automaton_state::{
            new_shared_automaton_state, AutomatonState, SharedAutomatonState
           }
    };
use automata_like_programming::automaton::{Automaton, NextState};
// Example implementation of an automaton state that appends specified text into
// mutable buffer.
pub struct TestState {
    id: u8,
    text: &'static str,
    next_state: Option<SharedAutomatonState<'static, u8, String, String>>
}

impl TestState {
    pub fn new(
        id: u8, 
        text: &'static str, 
        next_state: Option<SharedAutomatonState<'static, u8, String, String>>
    ) -> Self {
        Self { id, text, next_state }
    }
}

impl AutomatonState<'static, u8, String, String> for TestState {
    fn get_id_owned(
        &self
    ) -> u8 {
        self.id
    }
    
    fn get_id(
        &self
    ) -> &u8 {
        &self.id
    }
    
    fn execute_next_connection(
        &self, 
        data: &mut String
    ) -> Result<NextState<'static, u8, String, String>, String> {
        data.push_str(self.text);
        if let Option::Some(nxt_state) = &self.next_state {
            Result::Ok(NextState::Continue(Rc::clone(nxt_state)))
        } else {
            Result::Ok(NextState::NotFound)
        }
    }
}

let mut automaton = Automaton::new(|| {
    // First we create the "Bar" state as it's the last state and it doesn't connect to
    // any other state.
    let bar_state = new_shared_automaton_state(
                        TestState::new(2, "Bar", Option::None)
                    );
    // Secondly we declare the "Foo" state which is connected to "Bar" state.
    let foo_state = new_shared_automaton_state(
                        TestState::new(1, "Foo", Option::Some(Rc::clone(&bar_state)))
                    );
    foo_state
});
let mut buffer = String::with_capacity(6);
let result = automaton.run(&mut buffer);
assert!(result.is_could_not_find_next_state());
assert_eq!("FooBar", buffer);
```

## Example pattern matching with simple state implementation

This crate provides simple implementation of an automaton state. It uses keys provided by data passed to automaton when starting (e.g. data contains iterator). Each state has connections which define procedures for matching against given key, procedures to be executed when this connection is matched and next state that will become active. Below is an example of an automaton that finds "ab" pattern in given string.

```rust
use automata_like_programming::{
        automaton::{
            Automaton,
            AutomatonResult
        },    
        automaton_state::new_shared_concrete_state,
        simple_impl::simple_state::{
            KeyProvidingData,
            SimpleInterStateConnection,
            SimpleStateImplementation
        }
    };

 struct TextMatching<'a> {
    text: &'a str,
    matches: Vec<usize>,
    iter: usize,
}

impl <'a> TextMatching<'a> {
    pub fn new(
        text: &'a str
    ) -> Self {
        Self { text, matches: Vec::new(), iter: 0}
    }
    pub fn add_match(
        &mut self, index: usize
    ) -> () {
        self.matches.push(index);
    }
}

impl <'a> KeyProvidingData<(usize, char)> for TextMatching<'a> {
    fn next_key(
        &mut self
    ) -> Option<(usize, char)> {
        if self.iter >= self.text.len() {
            Option::None
        } else {
            self.iter += 1;
            Option::Some((self.iter - 1, self.text.chars().nth(self.iter)?))
        }
    }
}

fn char_matcher(
    c: char,
    reversed: bool
) -> impl Fn(&(usize, char)) -> bool {
    move |k| (k.1 == c) ^ reversed
}

let mut matching_data = TextMatching::new("aabbacacaabab");
let mut automaton: Automaton<u32, TextMatching, String> = Automaton::new(|| {
    let non_match_state = new_shared_concrete_state(SimpleStateImplementation::new(0));
    non_match_state.borrow_mut().register_connection(
        SimpleInterStateConnection::new_no_action(char_matcher('a', true), &non_match_state)
    );
    let a_state = new_shared_concrete_state(SimpleStateImplementation::new(1));
    non_match_state.borrow_mut().register_connection(
        SimpleInterStateConnection::new_no_action(char_matcher('a', false), &a_state)
    );
    a_state.borrow_mut().register_connection(
        SimpleInterStateConnection::new_no_action(char_matcher('a', false), &a_state)
    );
    a_state.borrow_mut().register_connection(
        SimpleInterStateConnection::new_no_action(char_matcher('b', true), &non_match_state)
    );
    
    let b_state = new_shared_concrete_state(SimpleStateImplementation::new(2));
    a_state.borrow_mut().register_connection(
        SimpleInterStateConnection::new(char_matcher('b', false),
        |data: &mut TextMatching, key| {
            data.add_match(key.0);
            Result::Ok(())
        }, &b_state)
    );
    b_state.borrow_mut().register_connection(
        SimpleInterStateConnection::new_no_action(char_matcher('a', false), &a_state)
    );
    b_state.borrow_mut().register_connection(
        SimpleInterStateConnection::new_no_action(char_matcher('a', true), &non_match_state)
    );
    non_match_state
});
let result = automaton.run(&mut matching_data);
assert!(result.is_empty_iter());
assert_eq!(matching_data.matches, vec![1, 9, 11]);
```
