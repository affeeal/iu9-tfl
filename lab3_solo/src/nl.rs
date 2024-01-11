#![allow(dead_code)]

mod extended_table;
mod main_table;

use std::collections::HashMap;

use crate::automata::{Automata, AutomataImpl, START};
use crate::config::{ALPHABET, EPSILON};
use crate::mat::{EquivalenceCheckResult, Mat};
use crate::nl::extended_table::ExtendedTable;
use crate::nl::main_table::MainTable;

// TODO: оптимизировать итерации в check_consistency

pub trait Nl {
    fn get_nfa(&mut self) -> Box<dyn Automata>;
}

pub struct NlImpl<'a> {
    mat: &'a dyn Mat,
    main_table: MainTable<'a>,
    extended_table: ExtendedTable<'a>,
}

impl<'a> Nl for NlImpl<'a> {
    fn get_nfa(&mut self) -> Box<dyn Automata> {
        loop {
            if let CompletenessCheckResult::UncoveredPrefix(prefix) = self.check_completeness() {
                self.insert_prefix(&prefix);
                continue;
            }

            if let ConsistencyCheckResult::DistinguishingSuffix(suffix) = self.check_consistency() {
                self.insert_suffix(&suffix);
                continue;
            }

            let nfa = self.build_nfa();

            if let EquivalenceCheckResult::Counterexample(word) =
                self.mat.check_equivalence(nfa.as_ref())
            {
                self.insert_prefix_recursive(&word);
                continue;
            }

            break nfa;
        }
    }
}

enum CompletenessCheckResult {
    Ok,
    UncoveredPrefix(String),
}

enum ConsistencyCheckResult {
    Ok,
    DistinguishingSuffix(String),
}

impl<'a> NlImpl<'a> {
    pub fn new(mat: &'a dyn Mat) -> Self {
        Self {
            mat,
            main_table: MainTable::new(mat),
            extended_table: ExtendedTable::new(mat),
        }
    }

    fn insert_prefix_recursive(&mut self, prefix: &str) {
        for i in 1..prefix.len() {
            self.insert_prefix(&prefix[0..i]);
        }
    }

    fn insert_prefix(&mut self, prefix: &str) {
        self.main_table.insert_prefix(prefix);
        self.extended_table.insert_prefix(prefix);
    }

    fn insert_suffix(&mut self, suffix: &str) {
        self.main_table.insert_suffix(suffix);
        self.extended_table.insert_suffix(suffix);
    }

    fn check_completeness(&self) -> CompletenessCheckResult {
        for prefix in &self.extended_table.prefixes {
            let membership_suffixes = self
                .main_table
                .prefix_to_membership_suffixes
                .get(prefix)
                .unwrap();
            if !self.main_table.is_covered(prefix, membership_suffixes) {
                return CompletenessCheckResult::UncoveredPrefix(prefix.to_owned());
            }
        }

        CompletenessCheckResult::Ok
    }

    fn check_consistency(&self) -> ConsistencyCheckResult {
        for (prefix_1, membership_suffixes_1) in &self.main_table.prefix_to_membership_suffixes {
            for (prefix_2, mebership_suffixes_2) in &self.main_table.prefix_to_membership_suffixes {
                if !membership_suffixes_1.is_subset(mebership_suffixes_2) {
                    continue;
                }

                for letter in ALPHABET.chars() {
                    let new_prefix_1 = format!("{prefix_1}{letter}");
                    let new_prefix_2 = format!("{prefix_2}{letter}");

                    let new_membership_suffixes_1 = self
                        .extended_table
                        .prefix_to_membership_suffixes
                        .get(&new_prefix_1)
                        .unwrap();
                    let new_membership_suffixes_2 = self
                        .extended_table
                        .prefix_to_membership_suffixes
                        .get(&new_prefix_2)
                        .unwrap();

                    if let Some(suffix) = new_membership_suffixes_1
                        .difference(new_membership_suffixes_2)
                        .next()
                    {
                        let distinguishing_suffix = format!("{letter}{suffix}");
                        return ConsistencyCheckResult::DistinguishingSuffix(distinguishing_suffix);
                    }
                }
            }
        }

        ConsistencyCheckResult::Ok
    }

    fn build_nfa(&self) -> Box<dyn Automata> {
        let mut automata = AutomataImpl::new(self.main_table.basic_prefixes.len() + 1);
        let prefix_to_index = self.enumerate_basic_prefixes();

        let epsilon_absorbed_prefixes = self.main_table.get_absorbed_basic_prefixes(
            self.main_table
                .prefix_to_membership_suffixes
                .get(EPSILON)
                .unwrap(),
        );
        for prefix in &epsilon_absorbed_prefixes {
            let index = prefix_to_index.get(prefix).unwrap();
            automata.transitions[START][*index] = Some(EPSILON.to_owned());
        }

        for (prefix, index) in &prefix_to_index {
            for letter in ALPHABET.chars() {
                let extension = format!("{prefix}{letter}");
                let extension_absorbed_prefixes = self.main_table.get_absorbed_basic_prefixes(
                    self.extended_table
                        .prefix_to_membership_suffixes
                        .get(&extension)
                        .unwrap(),
                );
                for absorbed_prefix in &extension_absorbed_prefixes {
                    let absorbed_prefix_index = prefix_to_index.get(absorbed_prefix).unwrap();
                    automata.transitions[*index][*absorbed_prefix_index] =
                        Some(letter.to_string());
                }
            }
        }

        let epsilon_membership_prefixes = self
            .main_table
            .suffix_to_membership_prefixes
            .get(EPSILON)
            .unwrap()
            .intersection(&self.main_table.basic_prefixes);
        for prefix in epsilon_membership_prefixes {
            let index = prefix_to_index.get(prefix).unwrap();
            automata.finite_states[*index] = true;
        }

        Box::new(automata)
    }

    fn enumerate_basic_prefixes(&self) -> HashMap<String, usize> {
        let mut prefix_to_index = HashMap::new();

        // Индекс 0 зарезервирован для стартового состояния
        for (i, prefix) in self.main_table.basic_prefixes.iter().enumerate() {
            prefix_to_index.insert(prefix.to_owned(), i + 1);
        }

        prefix_to_index
    }
}
