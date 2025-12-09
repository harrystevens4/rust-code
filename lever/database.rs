use std::fs::File;
use std::convert::TryInto;
use std::io;
use std::path::Path;
use std::io::Read;

pub struct LeverDB {
	installed_packages: Vec<(String,String)> //(name,repo_location)
}

impl LeverDB {
	pub fn load<T: AsRef<Path>>(path: T) -> io::Result<Self> {
		//defaults
		let mut installed_packages = vec![];
		//load file
		let mut database = String::new();
		let _ = File::open(path.as_ref())?.read_to_string(&mut database);
		//====== read the leverfile line by line ======
		let mut section_name = String::from("");
		for line in database.split('\n') {
			//skip empty lines
			if line.len() == 0 {continue}
			//sections
			if line.chars().next() == Some('[') {
				section_name = line[1..].trim_end_matches(']').into();
				continue;
			}
			match section_name.as_str() {
				"installed" => {
					//split by the deliminator "=>"
					let Ok([name,path]): Result<[&str;2],Vec<&str>> = line
						.split("=>")
						.collect::<Vec<_>>()
						.try_into() 
					else {continue}; //skip in line is invalid
					installed_packages.push((name.to_string(),path.to_string()));
				},
				_ => (),
			}
		}
		Ok(Self {
			installed_packages,
		})
	}
}
