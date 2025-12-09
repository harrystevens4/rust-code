mod leverfile;

use std::error::Error;
use leverfile::{LeverFile};
use std::env;
use std::io;

fn main() -> Result<(),Box<dyn Error>> {
	let command_line = env::args()
		.collect::<Vec<_>>()[1..]
		.to_owned();
	match command_line.iter().map(|s| s.as_str()).next() {
		Some("compile") => compile(command_line[1..].into()),
		Some("install") => install(command_line[1..].into()),
		Some("update") => update(command_line[1..].into()),
		Some("help") => Ok(help()),
		Some(command) => {
			eprintln!("Unknown command {command:?}");
			Err(Box::new(io::Error::other("bad argument")))
		},
		None => {
			eprintln!("Expected command as first argument.");
			Err(Box::new(io::Error::other("bad argument")))
		},
	}
}

fn compile(targets: Vec<String>) -> Result<(),Box<dyn Error>> {
	let leverfile = LeverFile::load("leverfile")?;
	leverfile.compile(".")?;
	Ok(())
}
fn install(targets: Vec<String>) -> Result<(),Box<dyn Error>> {
	let leverfile = LeverFile::load("leverfile")?;
	leverfile.install(".")?;
	Ok(())
}
fn update(targets: Vec<String>) -> Result<(),Box<dyn Error>> {
	for target in targets {
		compile(vec![target.clone()])?;
		install(vec![target.clone()])?;
	}
	Ok(())
}
fn help(){
	let name = env::args().next().expect("argv[0] nonexistent");
	println!("Lever is a git based package manager designed to help you manage compiling from source.");
	println!("");
	println!("Usage: {name} <command> [options]");
	println!("Commands:");
	println!("--> help");
	println!("Shows this help text. takes no arguments");
	println!("--> pull");
	println!("Pass it the name of packages to update via git pull. Passing nothing will cause it to update all.");
	println!("--> compile");
	println!("Pass it the name of packages to compile. Passing nothing will cause it to compile all.");
	println!("--> install");
	println!("Pass it the name of packages to install. Passing nothing will cause it to install all.");
	println!("--> update");
	println!("Pass it the name of packages to pull, compile then install. Passing nothing will cause it to act on all packages installed.");
}
