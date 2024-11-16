use crate::{automaton_state::new_shared_concrete_state, simple_impl::simple_state::{KeyProvidingData, SharedSimpleState, SimpleStateImplementation}};

use super::definer::SeriesDefiner;

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
/// #     series::builder::SeriesBuilder,
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
///   SeriesBuilder::new_without_default_action(SimpleStateImplementation::new('x'))
///   .next_state(char_matcher('a'), SimpleStateImplementation::new('a'))
///   .next_state(char_matcher('b'), SimpleStateImplementation::new('b'))
///   .next_state(char_matcher('c'), SimpleStateImplementation::new('c'))
///   .build()
/// }
/// ```
pub struct SeriesBuilder<'a, K, Id: Copy, D: KeyProvidingData<K>, E> {
    root: SharedSimpleState<'a, K, Id, D, E>,
    definer: SeriesDefiner<'a, K, Id, D, E>,
}

impl <'a, K, Id: Copy, D: KeyProvidingData<K>, E> SeriesBuilder<'a, K, Id, D, E> {
    pub fn new_without_default_action(root: SimpleStateImplementation<'a, K, Id, D, E>) -> SeriesBuilder<'a, K, Id, D, E> {
        let root_shared = new_shared_concrete_state(root);
        SeriesBuilder {
            definer: SeriesDefiner::new_without_default_action(&root_shared),
            root: root_shared,
        }
    }

    pub fn new<FDExec: Fn(&mut D, &K) -> Result<(), E> + 'a>(root: SimpleStateImplementation<'a, K, Id, D, E>, default_execution_function: FDExec) -> SeriesBuilder<'a, K, Id, D, E> {
        let root_shared = new_shared_concrete_state(root);
        SeriesBuilder {
            definer: SeriesDefiner::new(&root_shared, default_execution_function),
            root: root_shared,
        }
    }

    /// Creates new shared simple state and adds new connection to last added state with given matcher.
    pub fn next_state<M>(mut self, matcher: M, next_state: SimpleStateImplementation<'a, K, Id, D, E>) -> SeriesBuilder<'a, K, Id, D, E> 
    where M: 'a + Fn(&K) -> bool
    {
        let next_state_shared = new_shared_concrete_state(next_state);
        self.definer = self.definer.next_state(matcher, &next_state_shared);
        self
    }

    /// WARNING: Passed state will be modified after creating next connection, even before `build()` is called. Use with caution.
    /// Creates connection from last appended state to specified state.
    pub fn next_state_external<M>(mut self, matcher: M, next_state: &SharedSimpleState<'a, K, Id, D, E>) -> SeriesBuilder<'a, K, Id, D, E> 
    where M: 'a + Fn(&K) -> bool
    {
        self.definer = self.definer.next_state(matcher, next_state);
        self
    }

    /// Consumes builder and returns root state.
    pub fn build(self) -> SharedSimpleState<'a, K, Id, D, E> {
        self.root
    }
}