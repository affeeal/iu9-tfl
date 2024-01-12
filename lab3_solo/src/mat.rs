#![allow(dead_code)]

use std::process::Command;

use once_cell::sync::Lazy;

use crate::automata::Automata;

static mut LAST_TEST_WORD: usize = 0;
static mut TEST_WORDS: Lazy<Vec<String>> = Lazy::new(|| vec!["".to_string()]);

fn get_next_word(mat: &dyn Mat) -> String {
    unsafe {
        let next_word = TEST_WORDS[LAST_TEST_WORD].to_owned();
        LAST_TEST_WORD += 1;

        if LAST_TEST_WORD == TEST_WORDS.len() {
            let alphabet = mat.get_alphabet();
            let mut extension = Vec::with_capacity(TEST_WORDS.len() * alphabet.len());

            for word in TEST_WORDS.iter() {
                for letter in alphabet.chars() {
                    extension.push(format!("{word}{letter}"));
                }
            }

            TEST_WORDS.clear();
            TEST_WORDS.append(&mut extension);
            LAST_TEST_WORD = 0;
        }

        next_word
    }
}

pub enum EquivalenceCheckResult {
    Ok,
    Counterexample(String),
}

pub trait Mat {
    fn check_membership(&self, word: &str) -> bool;

    fn check_equivalence(&self, automata: &dyn Automata) -> EquivalenceCheckResult;

    fn get_alphabet(&self) -> String;
}

pub struct MatImpl {
    alphabet: String,
    test_words: Vec<String>,
    last_word: usize,
    max_tests: usize,
    oracle_path: String,
}

impl Mat for MatImpl {
    fn check_membership(&self, word: &str) -> bool {
        let output = Command::new(&self.oracle_path).arg(word).output().unwrap();
        String::from_utf8(output.stdout).unwrap().eq("1\n")
    }

    fn check_equivalence(&self, automata: &dyn Automata) -> EquivalenceCheckResult {
        for _ in 0..self.max_tests {
            let word = get_next_word(self);
            if self.check_membership(&word) && !automata.check_membership(&word) {
                return EquivalenceCheckResult::Counterexample(word);
            }
        }

        EquivalenceCheckResult::Ok
    }

    fn get_alphabet(&self) -> String {
        self.alphabet.to_string()
    }
}

impl MatImpl {
    pub fn new(alphabet: &str, max_tests: usize, oracle_path: &str) -> Self {
        Self {
            alphabet: alphabet.to_owned(),
            test_words: vec!["".to_string()],
            last_word: 0,
            max_tests,
            oracle_path: oracle_path.to_owned(),
        }
    }
}
