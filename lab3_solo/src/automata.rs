#![allow(dead_code)]

use std::any::Any;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

pub const EPSILON: &str = "";
pub const START: usize = 0;

pub trait Automata {
    fn as_any(&self) -> &dyn Any;

    fn check_membership(&self, word: &str) -> bool;

    fn determinize(&self) -> Box<dyn Automata>;

    fn to_regex(&self) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq)]
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
        let mut state = START;

        for letter in word.chars() {
            let letter = letter.to_string();

            let mut next_state = std::usize::MAX;
            for (i, label) in self.transitions[state].iter().enumerate() {
                if label.as_ref().eq(&Some(&letter)) {
                    next_state = i;
                    break;
                }
            }

            if next_state == std::usize::MAX {
                return false;
            }

            state = next_state;
        }

        self.finite_states[state]
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

    fn to_regex(&self) -> Option<String> {
        let mut automata = self.prepare_for_state_elimination();

        loop {
            let mut current = START;
            while current < automata.size
                && (automata.start_states[current] || automata.finite_states[current])
            {
                current += 1;
            }

            if current == automata.size {
                break match &automata.transitions[START].last().unwrap() {
                    Some(regex) => Some(regex.to_owned()),
                    None => None,
                };
            }

            for incoming in automata.get_incoming_states(current) {
                for outcoming in automata.get_outcoming_states(current) {
                    automata.eliminate_transition(incoming, current, outcoming);
                }
            }

            automata.eliminate_state(current);
        }
    }
}

impl AutomataImpl {
    pub fn new(size: usize) -> Self {
        let mut start_states = vec![false; size];
        start_states[START] = true;

        let transitions = vec![vec![None; size]; size];

        let finite_states = vec![false; size];

        Self {
            start_states,
            transitions,
            finite_states,
            size,
        }
    }

    fn prepare_for_state_elimination(&self) -> AutomataImpl {
        let mut automata = AutomataImpl::new(self.size + 1);

        for (i, row) in self.transitions.iter().enumerate() {
            for (j, letter) in row.iter().enumerate() {
                automata.transitions[i][j] = letter.to_owned();
            }

            if self.finite_states[i] {
                *automata.transitions[i].last_mut().unwrap() = Some(EPSILON.to_owned());
            }
        }

        *automata.finite_states.last_mut().unwrap() = true;

        automata
    }

    fn get_outcoming_states(&self, i: usize) -> Vec<usize> {
        let mut outcoming_states = Vec::<usize>::new();

        for j in 0..self.size {
            if self.transitions[i][j].is_some() && j != i {
                outcoming_states.push(j);
            }
        }

        outcoming_states
    }

    fn get_incoming_states(&self, i: usize) -> Vec<usize> {
        let mut incoming_states = Vec::<usize>::new();

        for j in 0..self.size {
            if self.transitions[j][i].is_some() && j != i {
                incoming_states.push(j);
            }
        }

        incoming_states
    }

    fn eliminate_transition(&mut self, incoming: usize, current: usize, outcoming: usize) {
        let former_regex_opt = &self.transitions[incoming][outcoming];
        let incoming_regex = self.transitions[incoming][current].as_ref().unwrap();
        let cyclic_regex_opt = &self.transitions[current][current];
        let outcoming_regex = self.transitions[current][outcoming].as_ref().unwrap();

        if Self::is_unfold_axiom_applicable(
            former_regex_opt,
            incoming_regex,
            cyclic_regex_opt,
            outcoming_regex,
        ) {
            self.transitions[incoming][outcoming] =
                Some(format!("{}*", Self::wrap_if_needed(incoming_regex)));
            return;
        }

        let mut result = String::new();

        let mut can_be_epsilon = false;
        let mut must_be_wrapped = false;

        if let Some(former_regex) = former_regex_opt {
            if former_regex.eq(&EPSILON) {
                can_be_epsilon = true;
            } else {
                must_be_wrapped = true;
                result.push_str(&format!("{former_regex}|"));
            }
        }

        result.push_str(incoming_regex);

        if let Some(cyclic_regex) = cyclic_regex_opt {
            result.push_str(&format!("{}*", Self::wrap_if_needed(cyclic_regex)));
        }

        if outcoming_regex.ne(&EPSILON) {
            result.push_str(outcoming_regex);
        }

        if must_be_wrapped {
            result = Self::wrap(&result);
        }

        if can_be_epsilon {
            result = format!("{}?", Self::wrap_if_needed(&result));
        }

        self.transitions[incoming][outcoming] = Some(result);
    }

    fn is_unfold_axiom_applicable(
        former_regex_opt: &Option<String>,
        incoming_regex: &String,
        cyclic_regex_opt: &Option<String>,
        outcoming_regex: &String,
    ) -> bool {
        *former_regex_opt == Some(EPSILON.to_owned())
            && Some(incoming_regex.clone()) == *cyclic_regex_opt
            && *outcoming_regex == EPSILON
    }

    fn wrap_if_needed(regex: &String) -> String {
        if regex.len() == 1
            || regex.chars().next() == Some('(') && regex.chars().last() == Some(')')
        {
            return regex.to_string();
        }

        Self::wrap(regex)
    }

    fn wrap(regex: &String) -> String {
        format!("({regex})")
    }

    fn eliminate_state(&mut self, i: usize) {
        self.start_states.swap_remove(i);

        self.transitions.swap_remove(i);
        for transition_row in &mut self.transitions {
            transition_row.swap_remove(i);
        }

        self.finite_states.swap_remove(i);

        self.size -= 1;
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
