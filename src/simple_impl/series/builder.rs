use crate::{automaton_state::new_shared_concrete_state, simple_impl::simple_state::{KeyProvidingData, SharedSimpleState, SimpleStateImplementation}};

use super::definer::SeriesDefiner;

/// Helper for creating automata structures. Designed to manage owned states (not referenced outside builder until `build()` is called), however it allows 
/// using states from outside of builder through functions marked as `external` (WARNING! Such states will be possibly modified on the fly in further calls
/// even if `build()` is not called).
pub struct SeriesBuilder<'a, K, Id: Copy, D: KeyProvidingData<K>, E> {
    root: SharedSimpleState<'a, K, Id, D, E>,
    definer: SeriesDefiner<'a, K, Id, D, E>,
}

impl <'a, K, Id: Copy, D: KeyProvidingData<K>, E> SeriesBuilder<'a, K, Id, D, E> {
    pub fn new(root: SimpleStateImplementation<'a, K, Id, D, E>) -> SeriesBuilder<'a, K, Id, D, E> {
        let root_shared = new_shared_concrete_state(root);
        SeriesBuilder {
            definer: SeriesDefiner::new(&root_shared),
            root: root_shared,
        }
    }

    pub fn next_connection<M>(mut self, matcher: M, next_state: SimpleStateImplementation<'a, K, Id, D, E>) -> SeriesBuilder<'a, K, Id, D, E> 
    where M: 'a + Fn(&K) -> bool
    {
        let next_state_shared = new_shared_concrete_state(next_state);
        self.definer = self.definer.next_connection(matcher, &next_state_shared);
        self
    }

    /// WARNING: Passed state will be modified after creating next connection, even before `build()` is called. Use with caution.
    /// Creates connection from last appended state to specified state.
    pub fn next_connection_external<M>(mut self, matcher: M, next_state: &SharedSimpleState<'a, K, Id, D, E>) -> SeriesBuilder<'a, K, Id, D, E> 
    where M: 'a + Fn(&K) -> bool
    {
        self.definer = self.definer.next_connection(matcher, next_state);
        self
    }

    /// Consumes builder and returns root state.
    pub fn build(self) -> SharedSimpleState<'a, K, Id, D, E> {
        self.root
    }
}