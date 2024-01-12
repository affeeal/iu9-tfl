use std::env;

// Язык bab*

fn main() {
    let args: Vec<String> = env::args().collect();
    let word = &args[1];

    let mut it = word.chars();

    if let Some(letter) = it.next() {
        if letter.ne(&'b') {
            println!("0");
            return;
        }
    }

    if let Some(letter) = it.next() {
        if letter.ne(&'a') {
            println!("0");
            return;
        }
    } else {
        println!("0");
        return;
    }

    while let Some(letter) = it.next() {
        if letter.ne(&'b') {
            println!("0");
            return;
        }
    }

    println!("1");
}
