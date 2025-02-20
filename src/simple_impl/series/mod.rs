/// Builder like implementation for creating linked sequences of states. Modifies passed
/// instances on the fly.
pub mod definer;
pub mod builder;

/// Simplifies creation of series by creating states based on provided iterator and connects them coninously starting from root.
/// Returns all states created in the process. Function can be passed to be executed at any default state switch (when no states
/// from series can be matched). Function to be executed while switching states can also be provided. Most useful for long patterns.
/// !!! WARNING Won't create returning connections for matched starting patterns (starting elements existing in further parts of
/// matched pattern meanining that new match could be possible while current match is still being processed).
/// # Examples
/// ```
/// use std::{cell::RefCell, rc::Rc};
/// use automata_like_programming::{
///     series,
///     automaton::Automaton,
///     automaton_state::new_shared_concrete_state,
///     simple_impl::simple_state::{
///         KeyProvidingData,
///         SimpleInterStateConnection,
///         SimpleStateImplementation,
///     }
/// };
/// # pub fn enumerated_char_matcher(c: char) -> impl Fn(&(usize, char)) -> bool {
/// #     move |k: &(usize, char)| k.1 == c
/// # }
/// # pub struct MatchListCopiedTestData<T> {
/// #     iter: usize,
/// #     values: Vec<T>,
/// #     matches: Vec<usize>,
/// # }
/// # 
/// # impl <T: Copy> MatchListCopiedTestData<T> {
/// #     pub fn new(values: Vec<T>) -> Self {
/// #         Self { iter: 0, values, matches: Vec::new() }
/// #     }
/// # 
/// #     pub fn set_match(&mut self, match_end: usize) -> () {
/// #         self.matches.push(match_end);
/// #     }
/// # 
/// #     pub fn matches(&self) -> &Vec<usize> {
/// #         &self.matches
/// #     }
/// # }
/// # 
/// # impl <T: Copy> KeyProvidingData<(usize, T)> for MatchListCopiedTestData<T> {
/// #     fn next_key(&mut self) -> Option<(usize,T)> {
/// #         let current_iter = self.iter;
/// #         let ret = self.values.get(self.iter);
/// #         self.iter += 1;
/// #         if let Some(c) = ret {
/// #             return Option::Some((current_iter,*c))
/// #         }
/// #         Option::None
/// #     }
/// # }
/// 
/// let mut automaton = Automaton::new({
/// let root: Rc<RefCell<SimpleStateImplementation<'_, (usize, char), u8, MatchListCopiedTestData<char>, String>>> = new_shared_concrete_state(SimpleStateImplementation::new(0));
/// let a2: Rc<RefCell<SimpleStateImplementation<'_, (usize, char), u8, MatchListCopiedTestData<char>, String>>> = new_shared_concrete_state(SimpleStateImplementation::new(4));
/// 
/// let defined_states = series!(
///                         // Iterator for generating states
///                         "abb".chars(),
///                         // Root node which will be the start of the series
///                         root,
///                         // Generator for matchers - creates matchers for enumerated elements from iterator
///                         enumerated_char_matcher,
///                         // Id generator - creates id values to be assigned to states
///                         |i, _| i as u8
///                      );
/// assert_eq!(defined_states.len(), 3);
/// defined_states.get(2).unwrap().borrow_mut().register_connection(
///     SimpleInterStateConnection::new(enumerated_char_matcher('a'),
///     |data, k| {
///         data.set_match(k.0);
///         Result::Ok(())
///     },
///     &a2));
/// defined_states.get(2).unwrap().borrow_mut().register_connection(
///     SimpleInterStateConnection::new_no_action(enumerated_char_matcher('a'), defined_states.get(0).unwrap())
/// );
/// a2.borrow_mut().register_connection(SimpleInterStateConnection::new_no_action_always_matched(&root));
/// root.borrow_mut().register_connection(SimpleInterStateConnection::new_no_action_always_matched(&root));
/// root
/// });
/// // Searched pattern starts at index 2 and ends on index 5 (abba).
/// let mut data = MatchListCopiedTestData::new("aaabbaba".chars().collect());
/// let automaton_result = automaton.run(&mut data);
/// assert!(automaton_result.is_empty_iter());
/// // Found one match.
/// assert_eq!(data.matches().len(), 1);
/// // Found match ending on 5th character.
/// assert_eq!(*data.matches().get(0).unwrap(), 5);
/// ```
#[macro_export]
macro_rules! series {
    ($iter:expr, $root:ident, $matcher_gen:ident, $id_gen:expr) => {
        $crate::series!($iter, $root, $matcher_gen, $id_gen, |_,_| {Result::Ok(())})
    };
    ($iter:expr, $root:ident, $matcher_gen:ident, $id_gen:expr, $default_exec:expr) => {
        $crate::series!($iter, $root, $matcher_gen, $id_gen, |_,_| {Result::Ok(())}, |_,_| {Result::Ok(())})
    };
    ($iter:expr, $root:ident, $matcher_gen:ident, $id_gen:expr, $default_exec:expr, $inter_state_exec:expr) => {
        {
            let mut defined_states = Vec::new();
            let mut definer = $crate::simple_impl::series::definer::SeriesDefiner::new(&$root, $default_exec);
            for x in $iter.enumerate() {
                let state = $crate::automaton_state::new_shared_concrete_state(SimpleStateImplementation::new(($id_gen)(x.0, x.1)));
                definer = definer.next_state_exec(($matcher_gen)(x.1), &state, $inter_state_exec);
                defined_states.push(state);
            }
            defined_states
        }
    };
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{automaton::Automaton, automaton_state::new_shared_concrete_state, simple_impl::simple_state::{SimpleInterStateConnection, SimpleStateImplementation}, test_commons::{enumerated_char_matcher, MatchListCopiedTestData}};

    #[test]
    fn series_from_string() -> () {
        let mut automaton = Automaton::new({
            let root: Rc<RefCell<SimpleStateImplementation<'_, (usize, char), u8, MatchListCopiedTestData<char>, String>>> = new_shared_concrete_state(SimpleStateImplementation::new(0));
            let a2: Rc<RefCell<SimpleStateImplementation<'_, (usize, char), u8, MatchListCopiedTestData<char>, String>>> = new_shared_concrete_state(SimpleStateImplementation::new(4));

            let defined_states = series!(
                                                                                // Iterator for generating states
                                                                                "abb".chars(),
                                                                                // Root node which will be the start of the series
                                                                                root,
                                                                                // Generator for matchers - creates matchers for enumerated elements from iterator
                                                                                enumerated_char_matcher,
                                                                                // Id generator - creates id values to be placed in states
                                                                                |i, _| i as u8);
            assert_eq!(defined_states.len(), 3);
            defined_states.get(2).unwrap().borrow_mut().register_connection(SimpleInterStateConnection::new(enumerated_char_matcher('a'), |data, k| {
                data.set_match(k.0);
                Result::Ok(())
            }, &a2));
            defined_states.get(2).unwrap().borrow_mut().register_connection(SimpleInterStateConnection::new_no_action(enumerated_char_matcher('a'), defined_states.get(0).unwrap()));
            a2.borrow_mut().register_connection(SimpleInterStateConnection::new_no_action_always_matched(&root));
            root.borrow_mut().register_connection(SimpleInterStateConnection::new_no_action_always_matched(&root));
            root
        });
        let mut data = MatchListCopiedTestData::new("aaabbaba".chars().collect());
        let automaton_result = automaton.run(&mut data);
        assert!(automaton_result.is_empty_iter());
        assert_eq!(data.matches().len(), 1);
        assert_eq!(*data.matches().get(0).unwrap(), 5);
    }
}