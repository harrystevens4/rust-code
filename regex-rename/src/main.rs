use std::env;
use std::io::Error;
use std::fs::rename;
use std::path::Path;
use std::process;
use regex;
fn main() -> Result<(),regex::Error>{
	let args = env::args().collect::<Vec<_>>();
	if args.len() != 4 {
		print_help();
		process::exit(1);
	}
	let replacement_name = &args[3];
	let regex_expr = &args[2];
	let file_path = &args[1];
	//====== compile regex ======
	let regex = regex::Regex::new(regex_expr)?;
	//====== check the file exists ======
	let file = Path::new(file_path);
	if !file.exists() {
		eprintln!("File does not exist");
		process::exit(2);//ENOENT
	}
	//====== figure out new name ======
	let name = file.file_name()
		.unwrap()
		.to_str()
		.unwrap();
	let new_name = regex.replace(name,replacement_name);
	//====== rename ======
	let new_path = file.parent().unwrap_or(Path::new(".")).join(Path::new(&new_name.into_owned()));
	println!("renaming {file:?} to {new_path:?}");
	rename(file,new_path).unwrap_or_else(|err|{
		eprintln!("Could not rename {file:?}: {err:?}");
		process::exit(Error::last_os_error()
			.raw_os_error()
			.unwrap()
		);
	});
	//return
	Ok(())
}
fn print_help(){
	let name = std::env::args().next().unwrap();
	println!("designed to be used in conjunction with the find command");
	println!("Usage: {name} [options] <filename> <match> <replacement>");
	println!("	where `match` is the regex to find and replace with `replacement`");
}
