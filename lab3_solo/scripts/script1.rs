use std::env;

// Язык a^n b^n, n > 0

fn main() {
    let args: Vec<String> = env::args().collect();
    let word: Vec<char> = (&args[1]).chars().collect();

    if word.len() % 2 == 1 {
        println!("0");
        return;
    }

    let mut left = 0;
    let mut right = word.len() - 1;

    while left < right {
        if word[left] == 'a' && word[right] == 'b' {
            left += 1;
            right -= 1;
        } else {
            println!("0");
            return;
        }
    }

    println!("1");
}
