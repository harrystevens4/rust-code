#![feature(trim_prefix_suffix)]
use args::Args;
use iniconfig::ConfigFile;
use std::collections::HashMap;
use std::fs;
use std::env;
use std::path::Path;
use std::process::{Command,ExitCode};

#[derive(Debug)]
struct Service {
	name: String,
	properties: HashMap<String,String>,
}

fn print_help(){
	println!("usage: {} [options]",env::args().next().unwrap_or("program".to_string()));
	println!("	-h, --help          : print help");
	println!("	-c, --config <path> : use config file at path provided");
}

fn main() -> ExitCode {
	let config_path;
	//====== read command line ======
	//skip argv[0]
	let cmd_line = env::args()
		.enumerate()
		.filter(|(i,_)| *i != 0)
		.map(|(_,x)| x)
		.collect::<Vec<_>>();
	let options = vec![
		(Some("h"),Some("help"),false),
		(Some("c"),Some("config"),true),
	];
	let args = match Args::new(cmd_line,options){
		Ok(args) => args,
		Err(e) => {
			eprintln!("FATAL: Error parsing arguments: {:?}",e);
			return ExitCode::FAILURE
		}
	};
	//====== apply command line options ======
	if args.has("h","help"){
		print_help();
		return ExitCode::SUCCESS
	}
	if args.has("c","config"){
		config_path = Path::new(args.get_arg(Some("c"),Some("config")).unwrap());
	}else {
		let mut default_config_path = match env::home_dir(){
			Some(dir) => dir,
			None => {
				eprintln!("FATAL: Cannot determine home directory.");
				return ExitCode::FAILURE
			}
		};
		default_config_path.push(".config/tmux-service-manager/startup.cfg");
		config_path = default_config_path.leak();
	}
	//====== read config file ======
	if !config_path.exists(){
		eprintln!("FATAL: Config file does not exist.");
		return ExitCode::FAILURE
	}
	let config_file = match fs::read_to_string(config_path){
		Ok(file) => file,
		Err(e) => {
			eprintln!("FATAL: error while reading config: {:?}",e);
			return ExitCode::FAILURE
		}
	};
	let config = ConfigFile::from(config_file.as_str());
	//println!("{:?}",config);
	//====== start tmuxes ======
	for section in config {
		let name = section.name();
		println!("Creating session \"{}\"...",name);
		println!("{:?}",section.properties());
		let mut args = vec!["new","-d","-s",name];
		if let Some(command) = section.properties().get("command"){
			args.push(command);
		}
		let mut command = Command::new("tmux");
			
		command.args(args);
		if let Some(cwd) = section.properties().get("cwd"){
			command.current_dir(cwd);
		}
		command.spawn();
	}

	//====== success ======
	ExitCode::SUCCESS
}

