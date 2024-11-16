use crate::simple_impl::simple_state::KeyProvidingData;

/// Returns matcher for character equal to the one provided as argument.
pub fn char_matcher(c: char) -> impl Fn(&char) -> bool {
    move |k: &char| *k == c
}

/// Returns matcher for character equal to the one provided as argument.
pub fn enumerated_char_matcher(c: char) -> impl Fn(&(usize, char)) -> bool {
    move |k: &(usize, char)| k.1 == c
}

/// Returns matcher for any character not equal to the one provided as argument.
pub fn char_matcher_reversed(c: char) -> impl Fn(&char) -> bool {
    move |k: &char| *k != c
}

/// Unwraps `Result` value without the need of implementing `Debug`. Panics with generic mesage.
/// For tests only.
macro_rules! unwrap_result {
    ($expression:expr) => {
        if let Ok(x) = $expression {
            x
        } else {
            panic!("Result was expected to be Ok(x) but was Err(x)");
        }
    };
}

pub(crate) use unwrap_result;

/// Provides key by repeating specified value for ever.
pub struct SingleValueTestData<T: Copy> {
    value: T,
}

impl <T: Copy> SingleValueTestData<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl <T: Copy> KeyProvidingData<T> for SingleValueTestData<T> {
    fn next_key(&mut self) -> Option<T> {
        Option::Some(self.value)
    }
}

/// Provide key by iterating over a vector of values that implement `Copy` trait.
pub struct CopiedTestData<T> {
    iter: usize,
    values: Vec<T>,
}

impl <T: Copy> CopiedTestData<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self { iter: 0, values }
    }
}

impl <T: Copy> KeyProvidingData<T> for CopiedTestData<T> {
    fn next_key(&mut self) -> Option<T> {
        let ret = self.values.get(self.iter);
        self.iter += 1;
        if let Some(c) = ret {
            return Option::Some(*c)
        }
        Option::None
    }
}

pub struct MatchListCopiedTestData<T> {
    iter: usize,
    values: Vec<T>,
    matches: Vec<usize>,
}

impl <T: Copy> MatchListCopiedTestData<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self { iter: 0, values, matches: Vec::new() }
    }

    pub fn set_match(&mut self, match_end: usize) -> () {
        self.matches.push(match_end);
    }

    pub fn matches(&self) -> &Vec<usize> {
        &self.matches
    }
}

impl <T: Copy> KeyProvidingData<(usize, T)> for MatchListCopiedTestData<T> {
    fn next_key(&mut self) -> Option<(usize,T)> {
        let current_iter = self.iter;
        let ret = self.values.get(self.iter);
        self.iter += 1;
        if let Some(c) = ret {
            return Option::Some((current_iter,*c))
        }
        Option::None
    }
}