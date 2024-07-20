//!
//! # Example of an automaton
//! 
//! ```
//! use std::rc::Rc;
//! use automata_like_programming::{
//!         automaton::AutomatonResult, automaton_state::{
//!             new_shared_automaton_state, AutomatonState, SharedAutomatonState
//!            }
//!     };
//!
//! use automata_like_programming::automaton::{Automaton, NextState};
//!
//! /// Example implementation of an automaton state that appends specified text into
//! /// mutable buffer.
//! pub struct TestState {
//!     id: u8,
//!     text: &'static str,
//!     next_state: Option<SharedAutomatonState<'static, u8, String>>
//! }
//! 
//! impl TestState {
//!     pub fn new(
//!         id: u8, 
//!         text: &'static str, 
//!         next_state: Option<SharedAutomatonState<'static, u8, String>>
//!     ) -> Self {
//!         Self { id, text, next_state }
//!     }
//! }
//! 
//! impl AutomatonState<'static, u8, String> for TestState {
//!     fn get_id_owned(
//!         &self
//!     ) -> u8 {
//!         self.id
//!     }
//!     
//!     fn get_id(
//!         &self
//!     ) -> &u8 {
//!         &self.id
//!     }
//!     
//!     fn execute_next_connection(
//!         &self, 
//!         data: &mut String
//!     ) -> Result<NextState<'static, u8, String>, String> {
//!         data.push_str(self.text);
//!         if let Option::Some(nxt_state) = &self.next_state {
//!             Result::Ok(NextState::Continue(Rc::clone(nxt_state)))
//!         } else {
//!             Result::Ok(NextState::NotFound)
//!         }
//!     }
//! }
//! 
//! let mut automaton = Automaton::new(|| {
//!     // First we create the "Bar" state as it's the last state and it doesn't connect to
//!     // any other state.
//!     let bar_state = new_shared_automaton_state(
//!                         TestState::new(2, "Bar", Option::None)
//!                     );
//!     // Secondly we declare the "Foo" state which is connected to "Bar" state.
//!     let foo_state = new_shared_automaton_state(
//!                         TestState::new(1, "Foo", Option::Some(Rc::clone(&bar_state)))
//!                     );
//!     foo_state
//! });
//! let mut buffer = String::with_capacity(6);
//! automaton.run(&mut buffer);
//! assert_eq!("FooBar", buffer);
//! ```
/// Basic part of automaton representing a node which is connected to either other nodes or itself.
pub mod automaton_state;
/// Core mechanism representing an automaton that travels through defined states.
pub mod automaton;
/// Simple implementations of automaton state.
pub mod simple_impl;