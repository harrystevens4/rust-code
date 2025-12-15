use std::fs::{File};
use LeverFile;
use std::convert::TryInto;
use std::io;
use std::path::{PathBuf,Path};
use std::io::Read;

pub struct LeverDB {
	pub installed_packages: Vec<(String,String)>, //(name,repo_location)
	pub db_path: PathBuf,
}

impl LeverDB {
	pub fn load<T: AsRef<Path>>(path: T) -> io::Result<Self> {
		//defaults
		let mut installed_packages = vec![];
		//load file
		let mut database = String::new();
		let _ = File::open(&path)?.read_to_string(&mut database);
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
					installed_packages.push((name.trim().to_string(),path.trim().to_string()));
				},
				_ => (),
			}
		}
		Ok(Self {
			installed_packages,
			db_path: path.as_ref().to_owned(),
		})
	}
	pub fn save(&self) -> io::Result<()>{
		let mut database_content = vec![];
		//====== installed packages ======
		database_content.push(String::from("[installed]"));
		for (name,location) in &self.installed_packages {
			database_content.push(name.to_owned() + "=>" + &location);
		}
		std::fs::write(&self.db_path,
			database_content
			.into_iter()
			.fold(String::new(),|string,line| string + &line + "\n") //fold into one long string
		)
	}
	pub fn add_installed<P: AsRef<Path>>(&mut self,path: P) -> Result<(),()> {
		let leverfile = LeverFile::load(&path)?;
		self.installed_packages.push((leverfile.name,path.as_ref().display().to_string()));
		Ok(())
	}
}
