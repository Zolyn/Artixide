use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sealed::sealed;

use crate::lazy;

lazy! {
    pub static FUZZY_MATCHER: SkimMatcherV2 = SkimMatcherV2::default();
}

#[sealed]
pub trait StrExt {
    fn fuzzy_indices(&self, choice: &str) -> Option<Vec<usize>>;
    fn slice(&self, start: usize, end: usize) -> Option<&str>;
}

#[sealed]
impl StrExt for str {
    fn fuzzy_indices(&self, choice: &str) -> Option<Vec<usize>> {
        FUZZY_MATCHER
            .fuzzy_indices(choice, self)
            .map(|(_, indices)| indices)
    }

    fn slice(&self, start: usize, end: usize) -> Option<&str> {
        if start >= end {
            return None;
        }

        let mut indices = self
            .char_indices()
            .map(|(index, _)| index)
            .chain(std::iter::once_with(|| self.len()));

        let start_index = indices.nth(start)?;

        let end_index = indices.nth(end - start - 1)?;

        Some(&self[start_index..end_index])
    }
}

#[sealed]
pub trait StringExt {
    fn take(&mut self) -> String;
}

#[sealed]
impl StringExt for String {
    fn take(&mut self) -> String {
        std::mem::take(self)
    }
}
