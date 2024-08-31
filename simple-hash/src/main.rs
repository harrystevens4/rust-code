use std::io;

fn main() {
    let mut text = String::new();
    let mut hash_string = String.new();
    io::stdin()
    	.read_line(&mut text)
    	.expect("Could not get stdin");	
    println!("Hashing {text}...");
    println!("hash: {hash_string}")
}
