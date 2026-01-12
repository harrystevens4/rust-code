use target_lexicon::Triple;
use faerie::artifact::*;
use args::Args;
use std::error::Error;
use std::fs::File;

fn main() -> Result<(),Box<dyn Error>> {
	//====== get file names from command line ======
	let args = Args::new(std::env::args().collect::<Vec<_>>()[1..].to_vec(),
		vec![
			(Some("h"),Some("help"),false),
			(Some("o"),Some("output"),true),
		]
	)?;
	let file_name = args.get_arg(Some("o"),Some("output"))
		.unwrap_or("data.o");
	let output_file = File::create(file_name)?;
	let file_names = args.other
		.clone();
	//====== start building our object file ====== 
	let mut object = ArtifactBuilder::new(Triple::host())
		.name(file_name.to_string())
		.finish();
	//====== declare symbols ======
	object.declarations(file_names
		.iter()
		.map(|f| (f,Decl::data().global().into()))
	)?;
	//====== define symbols ======
	for file_name in file_names {
		let file_size = 5_u64;
		let mut file_data = file_size.to_ne_bytes().to_vec();
		file_data.append(&mut "hello".as_bytes().to_vec());
		object.define(file_name,file_data)?
	}
	//====== write our new object ======
	object.write(output_file)?;
	Ok(())
}
