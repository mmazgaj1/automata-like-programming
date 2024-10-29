use std::rc::Rc;

use crate::simple_impl::simple_state::{KeyProvidingData, SharedSimpleState, SimpleInterStateConnection};

/// Allows for quick definition of chain of states that follow single path.
/// # Examples
/// ```
/// # use std::{cell::RefCell, rc::Rc};
/// # use automata_like_programming::{
/// #   automaton::{
/// #     AutomatonResult
/// #   }, 
/// #   automaton_state::{
/// #     new_shared_concrete_state,
/// #     AutomatonState
/// #   }, 
/// #   simple_impl::{
/// #     series::definer::SeriesDefiner,
/// #     simple_state::{
/// #       KeyProvidingData,
/// #       SimpleStateImplementation
/// #     }
/// #   }
/// # };
/// #
/// # fn char_matcher(c: char) -> impl Fn(&char) -> bool {
/// #   move |k: &char| *k == c
/// # }
/// #
/// # pub struct TestData {}
/// #
/// # impl KeyProvidingData<char> for TestData {
/// #   fn next_key(&mut self) -> Option<char> {
/// #     Option::Some('a')
/// #   }
/// # }
/// #
/// // Creates series of state that will match 'abc' character sequence or go back to root state anytime there is a mismatch.
/// fn create_abc_series_state_tree() -> Rc<RefCell<dyn AutomatonState<'static, char, TestData, String>>> {
///   let root_state = new_shared_concrete_state(SimpleStateImplementation::new('x'));
///   SeriesDefiner::new(&root_state)
///   .next_connection(char_matcher('a'), &new_shared_concrete_state(SimpleStateImplementation::new('a')))
///   .next_connection(char_matcher('b'), &new_shared_concrete_state(SimpleStateImplementation::new('b')))
///   .next_connection(char_matcher('c'), &new_shared_concrete_state(SimpleStateImplementation::new('c')));
///   root_state
/// }
/// ```
pub struct SeriesDefiner<'a, K, Id: Copy, D: KeyProvidingData<K>, E> {
    default_state: SharedSimpleState<'a, K, Id, D, E>,
    state: SharedSimpleState<'a, K, Id, D, E>,
}

impl <'a, K, Id: Copy, D: KeyProvidingData<K>, E> SeriesDefiner<'a, K, Id, D, E> {
    /// Creates new `SeriesDefiner` which defaults unmatched connections to the same node it starts from.
    pub fn new(starting_state: &SharedSimpleState<'a, K, Id, D, E>) -> SeriesDefiner<'a, K, Id, D, E> {
        Self::new_different_default_state(starting_state, starting_state)
    }

    /// Creates new `SeriesDefiner` with possible different starting and default states.
    pub fn new_different_default_state(starting_state: &SharedSimpleState<'a, K, Id, D, E>, default_state: &SharedSimpleState<'a, K, Id, D, E>) -> SeriesDefiner<'a, K, Id, D, E> {
        SeriesDefiner {
            state: Rc::clone(&starting_state),
            default_state: Rc::clone(&default_state),
        }
    }
    
    /// Adds connection to currently processed state. No function is executed when changing state. Creates a default connection 
    /// to starting state as well.
    pub fn next_connection<M>(mut self, matcher: M, next_state: &SharedSimpleState<'a, K, Id, D, E>) -> SeriesDefiner<'a, K, Id, D, E> 
    where M: 'a + Fn(&K) -> bool
    {
        self.state.borrow_mut().register_connection(SimpleInterStateConnection::new_no_action(matcher, next_state));
        self.state.borrow_mut().register_connection(SimpleInterStateConnection::new_no_action_always_matched(&self.default_state));
        self.state = Rc::clone(next_state);
        self
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::{automaton::{Automaton, AutomatonResult}, automaton_state::{new_shared_concrete_state, AutomatonState}, simple_impl::simple_state::{KeyProvidingData, SimpleStateImplementation}};

    use super::SeriesDefiner;

    struct TestData {
        iter: usize,
        chars: Vec<char>,
    }

    impl TestData {
        pub fn new(chars: Vec<char>) -> Self {
            Self { iter: 0, chars }
        }
    }

    impl KeyProvidingData<char> for TestData {
        fn next_key(&mut self) -> Option<char> {
            let ret = self.chars.get(self.iter);
            self.iter += 1;
            if let Some(c) = ret {
                return Option::Some(*c)
            }
            Option::None
        }
    }

    fn char_matcher(c: char) -> impl Fn(&char) -> bool {
        move |k: &char| *k == c
    }

    fn create_abc_series_state_tree() -> Rc<RefCell<dyn AutomatonState<'static, char, TestData, String>>> {
        let root_state = new_shared_concrete_state(SimpleStateImplementation::new('x'));
        SeriesDefiner::new(&root_state)
        .next_connection(char_matcher('a'), &new_shared_concrete_state(SimpleStateImplementation::new('a')))
        .next_connection(char_matcher('b'), &new_shared_concrete_state(SimpleStateImplementation::new('b')))
        .next_connection(char_matcher('c'), &new_shared_concrete_state(SimpleStateImplementation::new('c')));
        root_state
    }

    #[test]
    fn series_full_match() -> () {
        let mut automaton = Automaton::new(create_abc_series_state_tree);
        let mut data = TestData::new(vec!['a', 'b', 'c']);
        let automaton_result: AutomatonResult<char, String> = automaton.run(&mut data);
        assert!(automaton_result.is_empty_iter());
        if let AutomatonResult::EmptyIter(id) = automaton_result {
            assert_eq!(id, 'c');
        } else {
            panic!("Invalid result")
        }
    }

    #[test]
    fn series_back_to_default() -> () {
        let mut automaton = Automaton::new(create_abc_series_state_tree);
        let mut data = TestData::new(vec!['a', 'b', 'd']);
        let automaton_result: AutomatonResult<char, String> = automaton.run(&mut data);
        assert!(automaton_result.is_empty_iter());
        if let AutomatonResult::EmptyIter(id) = automaton_result {
            assert_eq!(id, 'x');
        } else {
            panic!("Invalid result")
        }
    }

    #[test]
    fn series_no_more_sates() -> () {
        let mut automaton = Automaton::new(create_abc_series_state_tree);
        let mut data = TestData::new(vec!['a', 'b', 'c', 'd']);
        let automaton_result: AutomatonResult<char, String> = automaton.run(&mut data);
        assert!(automaton_result.is_could_not_find_next_state());
        if let AutomatonResult::CouldNotFindNextState(id) = automaton_result {
            assert_eq!(id, 'c');
        } else {
            panic!("Invalid result")
        }
    }
}