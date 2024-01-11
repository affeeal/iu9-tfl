#![allow(dead_code)]

use std::any::Any;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::config::EPSILON;

pub const START: usize = 0;

pub trait Automata {
    fn as_any(&self) -> &dyn Any;

    fn check_membership(&self, word: &str) -> bool;

    fn determinize(&self) -> Box<dyn Automata>;
}

#[derive(Clone, PartialEq)]
pub struct AutomataImpl {
    pub size: usize,
    pub transitions: Vec<Vec<Option<String>>>,
    pub start_states: Vec<bool>,
    pub finite_states: Vec<bool>,
}

impl Automata for AutomataImpl {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn check_membership(&self, word: &str) -> bool {
        todo!()
    }

    fn determinize(&self) -> Box<dyn Automata> {
        #[derive(Eq, Hash, PartialEq)]
        struct Transition {
            from: usize,
            label: String,
            to: usize,
        }

        let start_subset = BTreeSet::from([START]);
        let mut subset_to_state = HashMap::from([(start_subset.to_owned(), START)]);
        let mut state_to_subset = HashMap::from([(START, start_subset)]);
        let mut state_counter = START + 1;
        let mut states_to_visit = VecDeque::from([START]);

        let mut finite_states = HashSet::<usize>::new();
        let mut transitions = HashSet::<Transition>::new();

        while let Some(state) = states_to_visit.pop_front() {
            let mut label_to_subset = HashMap::<String, BTreeSet<usize>>::new();
            let closure = self.get_epsilon_closure(state_to_subset.get(&state).unwrap());

            for closure_state in closure {
                if self.finite_states[closure_state] {
                    finite_states.insert(state);
                }

                for (next_state, label) in self.transitions[closure_state].iter().enumerate() {
                    if label.is_none() {
                        continue;
                    }

                    let label = label.as_ref().unwrap();
                    if label.eq(&EPSILON) {
                        continue;
                    }

                    if let Some(next_subset) = label_to_subset.get_mut(label) {
                        next_subset.insert(next_state);
                    } else {
                        label_to_subset.insert(label.to_owned(), BTreeSet::from([next_state]));
                    }
                }
            }

            for (label, next_subset) in &label_to_subset {
                let next_state: usize;

                if let Some(state) = subset_to_state.get(next_subset) {
                    next_state = *state;
                } else {
                    next_state = state_counter;
                    state_counter += 1;

                    subset_to_state.insert(next_subset.to_owned(), next_state);
                    state_to_subset.insert(next_state, next_subset.to_owned());
                    states_to_visit.push_back(next_state);
                }

                let transition = Transition {
                    from: state,
                    label: label.to_owned(),
                    to: next_state,
                };
                transitions.insert(transition);
            }
        }

        let mut automata = Self::new(state_counter);

        for transition in transitions {
            automata.transitions[transition.from][transition.to] =
                Some(transition.label.to_owned());
        }

        for state in finite_states {
            automata.finite_states[state] = true;
        }

        Box::new(automata)
    }
}

impl AutomataImpl {
    pub fn new(size: usize) -> Self {
        let mut start_states = vec![false; size];
        start_states[START] = true;

        let transition_matrix = vec![vec![None; size]; size];

        let finite_states = vec![false; size];

        Self {
            start_states,
            transitions: transition_matrix,
            finite_states,
            size,
        }
    }

    fn get_epsilon_closure(&self, subset: &BTreeSet<usize>) -> BTreeSet<usize> {
        let mut closure = BTreeSet::<usize>::new();

        for state in subset {
            closure.append(&mut self.get_state_epsilon_closure(*state));
        }

        closure
    }

    fn get_state_epsilon_closure(&self, state: usize) -> BTreeSet<usize> {
        let mut visited_states = BTreeSet::<usize>::new();
        let mut states_to_visit = VecDeque::from([state]);

        while let Some(state) = states_to_visit.pop_front() {
            visited_states.insert(state);

            for (next_state, label) in self.transitions[state].iter().enumerate() {
                if label.is_none() {
                    continue;
                }

                let label = label.as_ref().unwrap();
                if label.eq(&EPSILON) && !visited_states.contains(&next_state) {
                    states_to_visit.push_back(next_state);
                }
            }
        }

        visited_states
    }
}
