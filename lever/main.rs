mod leverfile;
mod database;

use leverfile::{LeverFile};
use std::path::{Path,PathBuf};
use database::LeverDB;
use std::env;
use std::process::exit;
use std::fs::read_to_string;

#[derive(Clone)]
pub struct Config {
	pub database_path: PathBuf
}

impl Config {
	pub fn new<P: AsRef<Path> + std::fmt::Debug + Clone>(config_path: Option<P>) -> Self {
		//====== initialise defaults ======
		let mut database_path = env::home_dir()
			.map(|p| p.join(".config/lever/packages.ini"))
			.unwrap_or(PathBuf::from("/etc/lever/packages.ini"));
		//====== load the config file if it exists ======
		let config_file_lines = config_path.clone()
			.map(|p| read_to_string(p).ok()) //read the config file and ignore errors
			.flatten()
			.map(|s| s
				.split('\n') //split file into lines
				.map(|x| match x.find('#') {
					Some(index) => x.chars().take(index).collect::<String>(), //remove comments
					None => x.to_string(),
				})
				.collect::<Vec<_>>() //wrap up into a nice Vec<String>
			);
		//====== read the config file if it exists ======
		if let Some(lines) = config_file_lines {
			for (line_number,line) in lines.into_iter().enumerate().filter(|(_,l)| l.len() != 0) {
				let Some((key,value)) = line.split_once('=') else {
					eprintln!("Malformed config in config file {:}:{}",config_path.as_ref().unwrap().as_ref().display(),line_number+1);
					continue
				};
				match key.trim() {
					"database_path" => {database_path = value.trim().into()},
					_ => eprintln!("Unknown config option {:?} in {:}:{}",key,config_path.as_ref().unwrap().as_ref().display(),line_number+1)
				}
			}
		}
		//====== return the final config ======
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
		Some("compile") => compile(command_line[1..].into(),&config),
		Some("install") => install(command_line[1..].into(),&config),
		Some("update") => update(command_line[1..].into(),&config),
		Some("track") => track(command_line[1..].into(),&config),
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

fn track(targets: Vec<String>, config: &Config) -> Result<(),()> {
	let database = match LeverDB::load(&config.database_path) {
		Ok(db) => db,
		Err(e) => {
			eprintln!("Error loading lever database ({:?}):",&config.database_path);
			eprintln!("{e:?}");
			return Err(());
		}
	};
	for file in targets {
		if !file.is_reg() || !file.exists() {
			eprintln!("{:?} is not a valid leverfile",file);
			continue;
		}
		LeverFile::load(file);
		database.add_installed(file);
	}
}

fn compile(targets: Vec<String>, config: &Config) -> Result<(),()> {
	let database = match LeverDB::load(&config.database_path) {
		Ok(db) => db,
		Err(e) => {
			eprintln!("Error loading lever database ({:?}):",&config.database_path);
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
			println!("Compiled {:?} without errors.\n",name);
		}
	}else{
		let leverfile = LeverFile::load("leverfile")?;
		leverfile.compile(".")?;
	}
	Ok(())
}
fn install(targets: Vec<String>, config: &Config) -> Result<(),()> {
	let leverfile = LeverFile::load("leverfile")?;
	println!("=== Installing {} ===",leverfile.name);
	leverfile.install(".")?;
	println!("Installed {:?} without errors.\n",leverfile.name);
	Ok(())
}
fn update(targets: Vec<String>, config: &Config) -> Result<(),()> {
	for target in targets {
		compile(vec![target.clone()],config)?;
		install(vec![target.clone()],config)?;
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
	println!("--> track");
	println!("Pass it the path of a leverfile(s) to track");
}
