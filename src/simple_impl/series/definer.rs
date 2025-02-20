use std::{rc::Rc, sync::Arc};

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
///   SeriesDefiner::new_without_default_action(&root_state)
///   .next_state(char_matcher('a'), &new_shared_concrete_state(SimpleStateImplementation::new('a')))
///   .next_state(char_matcher('b'), &new_shared_concrete_state(SimpleStateImplementation::new('b')))
///   .next_state(char_matcher('c'), &new_shared_concrete_state(SimpleStateImplementation::new('c')));
///   root_state
/// }
/// ```
pub struct SeriesDefiner<'a, K, Id: Copy, D: KeyProvidingData<K>, E> {
    default_state: SharedSimpleState<'a, K, Id, D, E>,
    default_execution: Arc<dyn Fn(&mut D, &K) -> Result<(), E> + 'a>,
    // default_execution: FDExec,
    state: SharedSimpleState<'a, K, Id, D, E>,
}

impl <'a, K, Id: Copy, D: KeyProvidingData<K>, E> SeriesDefiner<'a, K, Id, D, E> {
    /// Creates new `SeriesDefiner` which defaults unmatched connections to the same node it starts from.
    pub fn new_without_default_action(starting_state: &SharedSimpleState<'a, K, Id, D, E>) -> SeriesDefiner<'a, K, Id, D, E> {
        Self::new(starting_state, |_, _| {Result::Ok(())})
    }

    /// Creates new `SeriesDefiner` which defaults unmatched connections to the same node it starts from.
    pub fn new<FDExec: Fn(&mut D, &K) -> Result<(), E> + 'a>(starting_state: &SharedSimpleState<'a, K, Id, D, E>, default_execution_function: FDExec) -> SeriesDefiner<'a, K, Id, D, E> {
        Self::new_different_default_state(starting_state, starting_state, default_execution_function)
    }

    /// Creates new `SeriesDefiner` with possible different starting and default states.
    pub fn new_different_default_state<FDExec: Fn(&mut D, &K) -> Result<(), E> + 'a>(starting_state: &SharedSimpleState<'a, K, Id, D, E>, default_state: &SharedSimpleState<'a, K, Id, D, E>, default_execution_function: FDExec) -> SeriesDefiner<'a, K, Id, D, E> {
        SeriesDefiner {
            state: Rc::clone(&starting_state),
            default_state: Rc::clone(&default_state),
            default_execution: Arc::new(default_execution_function),
        }
    }
    
    /// Adds connection to currently processed state. No function is executed when changing state. Creates a default connection 
    /// to starting state as well.
    pub fn next_state<M>(mut self, matcher: M, next_state: &SharedSimpleState<'a, K, Id, D, E>) -> SeriesDefiner<'a, K, Id, D, E> 
    where M: 'a + Fn(&K) -> bool
    {
        self.state.borrow_mut().register_connection(SimpleInterStateConnection::new_no_action(matcher, next_state));
        let default_exec_function = Arc::clone(&self.default_execution);
        self.state.borrow_mut().register_connection(SimpleInterStateConnection::new_always_matched(move |k,d| (default_exec_function)(k, d), &self.default_state));
        self.state = Rc::clone(next_state);
        self
    }

    /// Adds connection to currently processed state. Given function is executed when changing state. Creates a default connection 
    /// to starting state as well.
    pub fn next_state_exec<M, FExec>(mut self, matcher: M, next_state: &SharedSimpleState<'a, K, Id, D, E>, execution_function: FExec) -> SeriesDefiner<'a, K, Id, D, E> 
    where
    M: 'a + Fn(&K) -> bool,
    FExec: 'a + Fn(&mut D, &K) -> Result<(), E>
    {
        self.state.borrow_mut().register_connection(SimpleInterStateConnection::new(matcher, execution_function, next_state));
        let default_exec_function = Arc::clone(&self.default_execution);
        self.state.borrow_mut().register_connection(SimpleInterStateConnection::new_always_matched(move |k,d| (default_exec_function)(k, d), &self.default_state));
        self.state = Rc::clone(next_state);
        self
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::{automaton::{Automaton, AutomatonResult}, automaton_state::{new_shared_concrete_state, AutomatonState}, simple_impl::simple_state::SimpleStateImplementation, test_commons::{char_matcher, unwrap_result, CopiedTestData}};

    use super::SeriesDefiner;

    fn create_abc_series_state_tree() -> Rc<RefCell<dyn AutomatonState<'static, char, CopiedTestData<char>, String>>> {
        let root_state = new_shared_concrete_state(SimpleStateImplementation::new('x'));
        SeriesDefiner::new(&root_state, |_, _| {Result::Ok(())})
        .next_state(char_matcher('a'), &new_shared_concrete_state(SimpleStateImplementation::new('a')))
        .next_state(char_matcher('b'), &new_shared_concrete_state(SimpleStateImplementation::new('b')))
        .next_state(char_matcher('c'), &new_shared_concrete_state(SimpleStateImplementation::new('c')));
        root_state
    }

    #[test]
    fn series_full_match() -> () {
        let mut automaton = Automaton::new(create_abc_series_state_tree());
        let mut data = CopiedTestData::new(vec!['a', 'b', 'c']);
        let automaton_result: AutomatonResult<char, String> = automaton.run(&mut data);
        assert_eq!(unwrap_result!(automaton_result.expect_empty_iter()), 'c');
    }

    #[test]
    fn series_back_to_default() -> () {
        let mut automaton = Automaton::new(create_abc_series_state_tree());
        let mut data = CopiedTestData::new(vec!['a', 'b', 'd']);
        let automaton_result: AutomatonResult<char, String> = automaton.run(&mut data);
        assert_eq!(unwrap_result!(automaton_result.expect_empty_iter()), 'x');
    }

    #[test]
    fn series_no_more_sates() -> () {
        let mut automaton = Automaton::new(create_abc_series_state_tree());
        let mut data = CopiedTestData::new(vec!['a', 'b', 'c', 'd']);
        let automaton_result = automaton.run(&mut data).expect_could_not_find_next_state();
        assert_eq!(unwrap_result!(automaton_result), 'c');
    }
}