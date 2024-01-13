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
    oracle_path: String,
    max_tests: usize,
    word_max_len: usize,
}

impl Mat for MatImpl {
    fn check_membership(&self, word: &str) -> bool {
        let output = Command::new(&self.oracle_path).arg(word).output().unwrap();
        String::from_utf8(output.stdout).unwrap().eq("1\n")
    }

    fn check_equivalence(&self, automata: &dyn Automata) -> EquivalenceCheckResult {
        for _ in 0..self.max_tests {
            let word = get_next_word(self);
            if word.len().ge(&self.word_max_len) {
                dbg!("Equivalence check has passed, because max_word_len has been reached");
                return EquivalenceCheckResult::Ok;
            }

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
    pub fn new(alphabet: &str, oracle_path: &str, max_tests: usize, word_max_len: usize) -> Self {
        Self {
            alphabet: alphabet.to_owned(),
            oracle_path: oracle_path.to_owned(),
            max_tests,
            word_max_len,
        }
    }
}
