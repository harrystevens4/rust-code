mod leverfile;

use std::error::Error;
use leverfile::LeverFile;

fn main() -> Result<(),Box<dyn Error>> {
	LeverFile::load("leverfile")?;
	Ok(())
}
