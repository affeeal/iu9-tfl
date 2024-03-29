pub mod automata;
pub mod mat;
pub mod nl;

use clap::Parser;
use nl::Nl;

use crate::automata::AutomataImpl;
use crate::mat::MatImpl;
use crate::nl::NlImpl;

#[derive(Parser)]
struct Cli {
    alphabet: String,
    oracle_path: String,
    max_tests: usize,
    word_max_len: usize,
}

fn main() {
    let args = Cli::parse();

    let mat = MatImpl::new(&args.alphabet, &args.oracle_path, args.max_tests, args.word_max_len);
    let mut nl = NlImpl::new(&mat);

    let dfa = nl.get_dfa();
    let dfa_impl = dfa.as_any().downcast_ref::<AutomataImpl>().unwrap();

    println!("{:?}", &dfa_impl);
}
