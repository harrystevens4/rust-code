use std::path::Path;
use std::os::unix::fs::MetadataExt;
use std::ffi::OsString;
use std::env;
use args::Args;
use args::ArgError::*;

#[link(name = "term")]
unsafe extern "C" {
	fn get_term_width() -> i32;
}

struct Config {
	colour: bool,
	max_depth: usize,
}

fn print_help(){
	println!("usage: {} [options] [path] ...",env::args().next().unwrap());
	println!("options:");
	println!("	-h, --help      : print help");
	println!("	-n, --no-colour : turn off colour");
	println!("	-d, --depth <n> : the maximum depth to look through");
}
fn main() {
	let mut config = Config {
		colour: true,
		max_depth: usize::MAX,
	};
	//====== parse arguments ======
	let format = vec![
	//       short      long               parameter
		(Some("h"), Some("help"),      false    ),
		(Some("n"), Some("no-colour"), false    ),
		(Some("d"), Some("depth"),     true     ),
	];
	let args = match Args::new(env::args().collect(),format){
		Ok(args) => args,
		Err(e) => match e {
			UnknownArgument(t) => {
				eprintln!("Error: unknown argument {:?}",t);
				return;
			},
			MissingParameter(t) => {
				eprintln!("Error: missing parameter to {:?}",t);
				return;
			}
		}
	};
	if args.has_short("h") || args.has_long("help") {
		print_help();
		return;
	}
	if args.has_short("n") || args.has_long("no-colour") {config.colour = false}
	if let Some(depth_arg) = args.get_arg(Some("d"),Some("depth")) {config.max_depth = depth_arg.parse().unwrap_or(usize::MAX)}
	//====== print_dir for each path given ======
	let mut dirs = args.other[1..].to_vec(); //exclude argv[0]
	if dirs.len() == 0 {dirs.push(".".into())}
	for arg in dirs{
		let dir = Path::new(&arg);
		println!("{}",match dir.file_name().unwrap_or(&OsString::from("..")).to_str(){
			Some(name) => name,
			None => {eprintln!("Path has bad unicode data"); continue},
		});
		if dir.exists() {print_dir(dir,"".into(),0,&config)}
	}
}
fn print_dir(path: &Path, indent: String, depth: usize, config: &Config){
	//dont excede max depth
	if depth > config.max_depth {return}
	//====== initialise things ======
	let width = unsafe {get_term_width()};
	let items = match path.read_dir(){
		Ok(d) => d,
		Err(e) => {eprintln!("{:?} - {:?}",path.file_name().unwrap_or(&OsString::from("..")),e); return},
	}.into_iter().collect::<Vec<_>>();
	if items.len() > 0 {
		//===== iterate over items in dir ======
		for (i,item) in items.iter().enumerate() {
			match item {
				Ok(item) => {
					//====== step into item if it is a dir ======
					let item_path = item.path();
					let file_name = match item.file_name().into_string(){
						Ok(name) => name,
						Err(e) => {eprintln!("{:?}: Bad unicode data",e); String::new()},
					};
					//====== colour filename if enabled ======
					let pretty_file_name = 
						if config.colour {format!("{}{}\x1b[39;49m",get_highlight(<&Path>::from(&item_path)),file_name)}
						else {file_name};
					//====== print the ones with ├ ======
					if i != items.len()-1 {print_clamped(format!("{}├─{}",&indent,pretty_file_name),width)}
					//====== the last one should use └ ======
					else {print_clamped(format!("{}└─{}",&indent,pretty_file_name),width)}
					if item_path.is_dir() {print_dir(&item_path.as_path(),indent.clone()+
						if i == items.len()-1 {"  "}
						else {"│ "}
					,depth+1,config)}
				},
				Err(e) => eprintln!("{:?}",e),
			}
		}
	}
}
fn print_clamped(string: String, width: i32){
	println!("{}",string.chars().take(width as usize).collect::<String>())
}
fn get_highlight(path: &Path) -> &str {
	if path.is_dir() {return "\x1b[34m"}
	if path.is_symlink() {return "\x1b[36m"}
	if let Ok(metadata) = path.metadata() {
		if metadata.mode() & 0o11 != 0 {return "\x1b[32m"}
	}
	""
}
