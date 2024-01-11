#![allow(dead_code)]

use crate::automata::Automata;

pub enum EquivalenceCheckResult {
    Ok,
    Counterexample(String),
}

pub trait Mat {
    fn check_membership(&self, word: &str) -> bool;

    fn check_equivalence(&self, automata: &dyn Automata) -> EquivalenceCheckResult;
}

pub struct MatImpl {
    // ...
}

impl Mat for MatImpl {
    fn check_membership(&self, word: &str) -> bool {
        todo!()
    }

    fn check_equivalence(&self, automata: &dyn Automata) -> EquivalenceCheckResult {
        todo!()
    }
}
