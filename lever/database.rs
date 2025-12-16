use std::fs::{File};
use std::io::Error;
use LeverFile;
use std::convert::TryInto;
use std::io;
use std::path::{PathBuf,Path};
use std::io::Read;

pub struct LeverDB {
	pub installed_packages: Vec<String>, //(name,repo_location)
	pub tracked_packages: Vec<(String,String)>, //(name,repo_location)
	pub db_path: PathBuf,
}

impl LeverDB {
	pub fn load<T: AsRef<Path>>(path: T) -> io::Result<Self> {
		//defaults
		let mut installed_packages = vec![];
		let mut tracked_packages = vec![];
		//create if it doesnt exist
		if !path.as_ref().exists() {File::create(&path)?;}
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
					//simply the name of the packages
					installed_packages.push(line.trim().to_string());
				},
				"tracked" => {
					//split by the deliminator "=>"
					let Ok([name,path]): Result<[&str;2],Vec<&str>> = line
						.split("=>")
						.collect::<Vec<_>>()
						.try_into() 
					else {continue}; //skip in line is invalid
					tracked_packages.push((name.trim().to_string(),path.trim().to_string()));
				},
				_ => (),
			}
		}
		Ok(Self {
			installed_packages,
			tracked_packages,
			db_path: path.as_ref().to_owned(),
		})
	}
	pub fn save(&self) -> io::Result<()>{
		let mut database_content = vec![];
		//====== tracked packages ======
		database_content.push(String::from("[tracked]"));
		for (name,location) in &self.tracked_packages {
			database_content.push(name.to_owned() + "=>" + &location);
		}
		//====== installed packages ======
		database_content.push(String::from("[installed]"));
		for name in &self.installed_packages {
			database_content.push(name.to_owned());
		}
		std::fs::write(&self.db_path,
			database_content
			.into_iter()
			.fold(String::new(),|string,line| string + &line + "\n") //fold into one long string
		)
	}
	pub fn get_package_location(&self,name_query: &str) -> Option<String> {
		self.tracked_packages.clone()
			.into_iter()
			.find(|(name,_)| name == name_query)
			.map(|(_,location)| location)
	}
	pub fn installed_packages(&self) -> Vec<String> {
		self.installed_packages.clone()
	}
	pub fn add_tracked<P: AsRef<Path>>(&mut self,path: P) -> io::Result<()> {
		let leverfile = LeverFile::load(&path).map_err(|_| Error::other("Could not load leverfile"))?;
		if self.tracked_packages
			.iter()
			.any(|(name,_)| *name == leverfile.name){
				return Err(Error::other("package already tracked"))
		}
		self.tracked_packages.push((leverfile.name,path.as_ref().display().to_string()));
		Ok(())
	}
	pub fn add_installed(&mut self,package_name: &str) -> io::Result<()> {
		if self.installed_packages
			.iter()
			.any(|name| *name == package_name){
				return Err(Error::other("package already installed"))
		}
		self.installed_packages.push(package_name.to_owned());
		Ok(())
	}
}
