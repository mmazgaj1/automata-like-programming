//! # Example of automaton implementation for finding "ab" pattern
//! 
//! ```
//! use automata_like_programming::{
//!         automaton::Automaton,
//!         automaton_state::new_shared_concrete_state,
//!         simple_impl::simple_state::
//!         {
//!             KeyProvidingData,
//!             SimpleInterStateConnection,
//!             SimpleStateImplementation
//!         }
//! };
//! 
//!  struct TextMatching<'a> {
//!     text: &'a str,
//!     matches: Vec<usize>,
//!     iter: usize,
//! }
//! 
//! impl <'a> TextMatching<'a> {
//!     pub fn new(
//!         text: &'a str
//!     ) -> Self {
//!         Self { text, matches: Vec::new(), iter: 0}
//!     }
//!     pub fn add_match(
//!         &mut self, index: usize
//!     ) -> () {
//!         self.matches.push(index);
//!     }
//! }
//! 
//! impl <'a> KeyProvidingData<(usize, char)> for TextMatching<'a> {
//!     fn next_key(
//!         &mut self
//!     ) -> Option<(usize, char)> {
//!         if self.iter >= self.text.len() {
//!             Option::None
//!         } else {
//!             self.iter += 1;
//!             Option::Some((self.iter - 1, self.text.chars().nth(self.iter)?))
//!         }
//!     }
//! }
//! 
//! fn char_matcher(
//!     c: char,
//!     reversed: bool
//! ) -> impl Fn(&(usize, char)) -> bool {
//!     move |k| (k.1 == c) ^ reversed
//! }
//! 
//! let mut matching_data = TextMatching::new("aabbacacaabab");
//! let mut automaton: Automaton<u32, TextMatching> = Automaton::new(|| {
//!     let non_match_state = new_shared_concrete_state(SimpleStateImplementation::new(0));
//!     non_match_state.borrow_mut().register_connection(
//!         SimpleInterStateConnection::new_no_action(char_matcher('a', true), &non_match_state)
//!     );
//!
//!     let a_state = new_shared_concrete_state(SimpleStateImplementation::new(1));
//!     non_match_state.borrow_mut().register_connection(
//!         SimpleInterStateConnection::new_no_action(char_matcher('a', false), &a_state)
//!     );
//!     a_state.borrow_mut().register_connection(
//!         SimpleInterStateConnection::new_no_action(char_matcher('a', false), &a_state)
//!     );
//!     a_state.borrow_mut().register_connection(
//!         SimpleInterStateConnection::new_no_action(char_matcher('b', true), &non_match_state)
//!     );
//!     
//!     let b_state = new_shared_concrete_state(SimpleStateImplementation::new(2));
//!     a_state.borrow_mut().register_connection(
//!         SimpleInterStateConnection::new(char_matcher('b', false),
//!         |data: &mut TextMatching, key| {
//!             data.add_match(key.0);
//!             Result::Ok(())
//!         }, &b_state)
//!     );
//!     b_state.borrow_mut().register_connection(
//!         SimpleInterStateConnection::new_no_action(char_matcher('a', false), &a_state)
//!     );
//!     b_state.borrow_mut().register_connection(
//!         SimpleInterStateConnection::new_no_action(char_matcher('a', true), &non_match_state)
//!     );
//!     non_match_state
//! });
//! automaton.run(&mut matching_data);
//! assert_eq!(matching_data.matches, vec![1, 9, 11]);
//! ```


/// Basic implementation of an automaton state. Provides management for handling connections between
/// states and allows for some action to be executed while changing states. Designed to be used
/// in parser like solutions.
pub mod simple_state;
