const ROCK_ASCII: &str = 
"
           __ __
      _-__/  -  \\
     /       -   |
    |  ..        |
    |  .         \\
    |        --  /
    \\  -     -  /
     |      ___/
     \\__-__/
";
const PAPER_ASCII: &str = 
"
       ____________
      /|          |
     / |~~~~~     |
    /  |          |
   /   |~~~~~~~   |
  /    |          |
 /-----/~~~~~     |
 |                |
 | ~~~~~~~~~      |
 |                |
 | ~~~~~~~~~~~~   |
 |                |
 |----------------|
";
const SCISSORS_ASCII: &str =
"
           ____
    ____  /    \\
   /    \\ |    |
   |    | \\____/
   \\____/ o
          |\\
          | \\
          |  \\
          |   \\
          |    \\
          |     \\
          |
";
const ART: [&str; 3] = [
	ROCK_ASCII,PAPER_ASCII,SCISSORS_ASCII
];

use std::io;
fn main() -> io::Result<()>{
	use rand::{Rng,rng};
    println!("Enter rock, paper or scissors");
	loop {
		let computer_choice: u8 = rng().random_range(0..3);
		let mut line = String::new();
		io::stdin().read_line(&mut line)?;
		match line.as_str().trim() {
			"rock"     | "r" => compare_win(0,computer_choice),
			"paper"    | "p" => compare_win(1,computer_choice),
			"scissors" | "s" => compare_win(2,computer_choice),
			"" => break Ok(()),
			_ => {
				eprintln!("unrecognised word {:?}",line.trim());
				continue;
			},
		}
	}
}
fn compare_win(player_choice: u8, computer_choice: u8){
	println!("====== You chose ======\n{}",ART[player_choice as usize]);
	println!("====== I chose ======\n{}",ART[computer_choice as usize]);
	print!("---> ");
	if player_choice == (computer_choice+1)%3 {println!("You win")}
	else if player_choice == computer_choice {println!("Draw")}
	else {println!("You lose")}
}
