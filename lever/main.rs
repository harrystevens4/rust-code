mod leverfile;
mod database;

use leverfile::{LeverFile};
use std::path::{Path,PathBuf};
use database::LeverDB;
use std::env;
use std::process::exit;
use std::fs::read_to_string;

#[derive(Clone)]
struct Config {
	database_path: PathBuf
}

impl Config {
	fn new<P: AsRef<Path>>(config_path: Option<P>) -> Self {
		let mut database_path = env::home_dir()
			.map(|p| p.join(".config/lever/packages.ini"))
			.unwrap_or(PathBuf::from("/etc/lever/packages.ini"));
		let config_file_lines = config_path
			.map(|p| read_to_string(p).ok()) //read the config file and ignore errors
			.flatten()
			.map(|s| s
				.split('\n') //split file into lines
				.filter(|x| x.len() != 0) //cut empty lines
				.map(|x| x.to_string()) //gain ownership
				.collect::<Vec<_>>() //wrap up into a nice Vec<String>
			);
		if let Some(lines) = config_file_lines {
			for line in lines {
				println!("{}",line);
			}
		}
		Self {
			database_path,
		}
	}
}

fn main(){
	let config_path = env::home_dir()
		.map(|p| p.join(".config/lever/lever.conf"))
		.unwrap_or(PathBuf::from("/etc/lever/lever.conf"));
	let config = Config::new(Some(config_path));
	let command_line = env::args()
		.collect::<Vec<_>>()[1..]
		.to_owned();
	let Ok(()) = (match command_line.iter().map(|s| s.as_str()).next() {
		Some("compile") => compile(command_line[1..].into(),config),
		Some("install") => install(command_line[1..].into(),config),
		Some("update") => update(command_line[1..].into(),config),
		Some("help") => Ok(help()),
		Some(command) => {
			eprintln!("Unknown command {command:?}");
			Err(())
		},
		None => {
			eprintln!("Expected command as first argument.");
			Err(())
		},
	}) else {exit(1)};
}

fn compile(targets: Vec<String>, config: Config) -> Result<(),()> {
	let database = match LeverDB::load("lever.db") {
		Ok(db) => db,
		Err(e) => {
			eprintln!("{e:?}");
			return Err(());
		}
	};
	if targets.len() == 0 {
		//select all
		for (name, location) in database.installed_packages {
			println!("=== Compiling {} ===",name);
			let path = Path::new(&location);
			let Ok(leverfile) = LeverFile::load(path) else {
				eprintln!("Loading leverfile at {path:?} failed.");
				return Err(())
			};
			leverfile.compile(path)?;
			println!("Compiled {:?} without errors.",name);
		}
	}else{
		let leverfile = LeverFile::load("leverfile")?;
		leverfile.compile(".")?;
	}
	Ok(())
}
fn install(targets: Vec<String>, config: Config) -> Result<(),()> {
	let leverfile = LeverFile::load("leverfile")?;
	println!("=== Installing {} ===",leverfile.name);
	leverfile.install(".")?;
	println!("Installed {:?} without errors.",leverfile.name);
	Ok(())
}
fn update(targets: Vec<String>, config: Config) -> Result<(),()> {
	for target in targets {
		compile(vec![target.clone()],config.clone())?;
		install(vec![target.clone()],config.clone())?;
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
