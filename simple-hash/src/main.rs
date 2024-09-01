use std::io;

fn main() {
    let hash_length = 10;
    let mut text = String::new();
    let mut hash_string = String::new();
    io::stdin()
    	.read_line(&mut text)
    	.expect("Could not get stdin");	
    text.truncate(text.len() - 1);//remove trailing newline
    println!("Hashing {text}");
    let text_length = text.len();
    let mut text_index = 0;
    for hash_index in 0..hash_length {
    	println!("copying {:?} to {hash_index}",text.chars().nth(text_index));
	let mut current_char: char = text
	    .chars()
	    .nth(text_index)
	    .expect("could not index string");
	hash_string.push(current_char);
	text_index += 1;
	if text_index == text_length { //wrap around to 0
	    text_index = 0;
	}
    }
    println!("hash: {hash_string}");
}
