mod leverfile;

use std::error::Error;
use leverfile::{LeverFile};

fn main() -> Result<(),Box<dyn Error>> {
	let leverfile = LeverFile::load("leverfile")?;
	println!("{:?}",leverfile);
	leverfile.compile(".")?;
	Ok(())
}
