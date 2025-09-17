#![feature(trim_prefix_suffix)]
use args::Args;
use std::collections::HashMap;
use std::fs;
use std::env;
use std::path::Path;
use std::process::ExitCode;

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
	let config = parse_config(&config_file);
	println!("{:?}",config);

	//====== success ======
	ExitCode::SUCCESS
}

fn parse_config(raw_config: &str) -> Vec<Service>{
	let config_data: Vec<Service> = vec![];
	let lines = raw_config.split('\n').collect::<Vec<&str>>();
	let mut services = read_label("top_level",lines.into_iter());
	//the first "service" will be everything before the first label, so strip it off
	services.remove(0);
	services
}
fn is_label(line: &str) -> bool {
	if line.trim().chars().nth(0) == Some('[') {return true}
	false
}
fn extract_label(line: &str) -> String {
	line.trim().trim_prefix("[").trim_suffix("]").into()
}
fn read_label<'a>(name: &str, mut lines: impl Iterator<Item = &'a str>) -> Vec<Service>{
	let mut service = Service {
		properties: HashMap::new(),
		name: name.into(),
	};
	//====== for line in lines ======
	loop {
		//the flipping for loop calls into_iterator which moves. SO ANOYING
		let line = match lines.next() { Some(v) => v, None => break, };
		//====== if it is a valid property, add it to the hashmap ======
		if is_property(line){
			let (key, value) = extract_property(line);
			let _ = service.properties.insert(key,value);
		}
		//====== recurse if another label is reached ======
		if is_label(line){
			let service_name = extract_label(line);
			let mut result = vec![service];
			result.append(&mut read_label(&service_name,lines));
			return result
		}
	}
	//base case
	vec![service]
}
fn is_property(line: &str) -> bool {
	line.contains('=')
}
fn extract_property(line: &str) -> (String,String){
	let result = line.split('=').take(2).collect::<Vec<_>>();
	(result[0].into(), result[1].into())
}
