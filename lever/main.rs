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
	let Ok(_) = (match command_line.iter().map(|s| s.as_str()).next() {
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
	let mut database = match LeverDB::load(&config.database_path) {
		Ok(db) => db,
		Err(e) => {
			eprintln!("Error loading lever database ({:?}):",&config.database_path);
			eprintln!("{e:?}");
			return Err(());
		}
	};
	for file in targets {
		if !Path::new(&file).is_file() || !Path::new(&file).exists() {
			eprintln!("{:?} is not a valid leverfile",file);
			continue;
		}
		let _ = database.add_tracked(file).map_err(|e| eprintln!("Error tracking in database: {e:?}"))?;
	}
	database.save().map_err(|e| {
		eprintln!("Error saving database: {e:?}");
		() //remove the Err content as we have already printed it
	})
}

fn compile(targets: Vec<String>, config: &Config) -> Result<(),()> {
	//====== load the package database ======
	let database = match LeverDB::load(&config.database_path) {
		Ok(db) => db,
		Err(e) => {
			eprintln!("Error loading lever database ({:?}):",&config.database_path);
			eprintln!("{e:?}");
			return Err(());
		}
	};
	let compile_queue = if targets.len() == 0 {database.installed_packages()}
		else {targets};
	//====== compile all the selected packages ======
	for package in compile_queue {
		//get the path to the leverfile
		let Some(leverfile_path) = database.get_package_location(&package)
		else {
			eprintln!("Could not find package {package:?}, skipping");
			continue;
		};
		//load the leverfile
		println!("=== Compiling {} ===",package);
		let Ok(leverfile) = LeverFile::load(&leverfile_path) else {
			eprintln!("Loading leverfile at {leverfile_path:?} failed.");
			return Err(())
		};
		//determine the compile dir
		let Some(compile_dir) = Path::new(&leverfile_path).parent()
		else {
			eprintln!("Could not determine compile directory.");
			return Err(());
		};
		//actualy compile
		match leverfile.compile(compile_dir) {
			Ok(_) => (),
			Err(e) => {
				eprintln!("Compilation error: {e:?}");
				return Err(());
			}
		};
		println!("Compiled {:?} without errors.\n",package);
	}
	Ok(())
}
fn install(targets: Vec<String>, config: &Config) -> Result<(),()> {
	//====== load the database ======
	let mut database = match LeverDB::load(&config.database_path) {
		Ok(db) => db,
		Err(e) => {
			eprintln!("Error loading lever database ({:?}):",&config.database_path);
			eprintln!("{e:?}");
			return Err(());
		}
	};
	//====== install all if no targets are provided ======
	let mut install_queue = targets.clone();
	if targets.len() == 0 {
		let mut all_installed_packages = database.installed_packages();
		install_queue.append(&mut all_installed_packages);
	}
	//====== install selected packages ======
	for package in install_queue {
		//get the leverfile path
		let Some(leverfile_path) = database.get_package_location(&package)
		else {
			eprintln!("Could not find package {package:?}, skipping");
			continue;
		};
		println!("=== Installing {} ===",package);
		//load the leverfile
		let Ok(leverfile) = LeverFile::load(&leverfile_path) else {
			eprintln!("Loading leverfile at {leverfile_path:?} failed.");
			return Err(())
		};
		//determine the compile dir
		let Some(compile_dir) = Path::new(&leverfile_path).parent()
		else {
			eprintln!("Could not determine compile directory.");
			return Err(());
		};
		//actualy compile
		match leverfile.install(compile_dir) {
			Ok(_) => (),
			Err(e) => {
				eprintln!("Install error: {e:?}");
				return Err(());
			}
		};
		println!("Installed {:?} without errors.\n",package);
		//track that the package has now been installed
		if let Ok(_) = database.add_installed(&package) {
			let _ = database.save();
		}
	}
	//TODO: add newly installed targets to installed section of db
	//TODO: only install targets if specified
	//TODO: handle git clone if path to leverfile provided
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
	println!("Pass it the name of packages to update via git pull. Passing nothing will cause it to update all installed packages.");
	println!("--> compile");
	println!("Pass it the name of packages to compile. Passing nothing will cause it to compile all installed packages.");
	println!("--> install");
	println!("Pass it the name of packages to install. Passing nothing will cause it to reinstall all already installed packages.");
	println!("Passing a path to a leverfile will cause it to move it to the default folder, clone the repo, track it and then install it");
	println!("--> update");
	println!("Pass it the name of packages to pull, compile then install. Passing nothing will cause it to act on all packages installed.");
	println!("--> track");
	println!("Pass it the path of a leverfile(s) to track");
}
