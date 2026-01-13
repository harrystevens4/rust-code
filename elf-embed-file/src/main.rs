use target_lexicon::Triple;
use std::path::Path;
use std::ffi::OsStr;
use faerie::artifact::*;
use args::Args;
use std::error::Error;
use std::fs::{File,read};

fn main() -> Result<(),Box<dyn Error>> {
	//====== get file names from command line ======
	let args = Args::new(std::env::args().collect::<Vec<_>>()[1..].to_vec(),
		vec![
			(Some("h"),Some("help"),false),
			(Some("o"),Some("output"),true),
			(Some("n"),Some("name-only"),false),
		]
	)?;
	if args.has("h","help") {
		print_help();
		return Ok(());
	}
	let name_only = args.has("n","name-only");
	let file_name = args.get_arg(Some("o"),Some("output"))
		.unwrap_or("data.o");
	let output_file = match File::create(file_name){
		Ok(f) => f,
		Err(e) => {
			eprintln!("Could not open output file {file_name:?}: {e}");
			return Err(e.into());
		}
	};
	let file_names = args.other
		.clone();
	let symbol_names = file_names
		.clone()
		.into_iter()
		.map(|path| if name_only {
			//extract the filename from the path
			//only returns none if filename is ".."
			sanitise_symbol_name(
				Path::new(&path)
				.file_name()
				.unwrap_or(OsStr::new(".."))
				.to_string_lossy()
				.into_owned()
			)
		} else {sanitise_symbol_name(path)})
		.collect::<Vec<_>>();
	//====== start building our object file ====== 
	let mut object = ArtifactBuilder::new(Triple::host())
		.name(file_name.to_string())
		.finish();
	//====== declare symbols ======
	//data symbols
	object.declarations(symbol_names
		.iter()
		.map(|f| f.to_string()) //change '/' to '_'
		.map(|f| (f,Decl::data().global().into()))
	)?;
	//data size symbols
	object.declarations(symbol_names
		.iter()
		.map(|f| f.to_string() + "_size")
		.map(|f| (f,Decl::data().global().into()))
	)?;
	//====== define symbols ======
	for (file_name,symbol_name) in file_names.into_iter().zip(symbol_names) {
		let file_data = read(&file_name)?;
		let file_size = file_data.len() as u64;
		//define the data and size symbols
		object.define(symbol_name.clone() + "_size",file_size.to_ne_bytes().to_vec())?;
		object.define(symbol_name,file_data)?;
	}
	//====== write our new object ======
	object.write(output_file)?;
	Ok(())
}

fn sanitise_symbol_name(name: String) -> String {
	name
		.replace("/","_") //replace these with '_'
		.replace("-","_")
		.replace(".","_")
		.replace(" ","_")
		.chars()
		//delete any other non alphanumeric characters
		.filter(|c| (c.is_alphanumeric() || *c == '_') && c.is_ascii())
		.collect()
}

fn print_help(){
	let name = std::env::args().next().expect("No argv[0] found");
	println!("Usage: {name} [options] <file 1> ... <file n>");
	println!("Options:");
	println!("	-o, --output <name> : change the name of the output file. defaults to data.o");
	println!("	-h, --help          : display this help message");
	println!("	-n, --name-only     : Do not use the full path (\"/usr/bin/a\" etc), only use the filename when generating symbol names (\"/usr/bin/a\" becomes \"a\")");
}
