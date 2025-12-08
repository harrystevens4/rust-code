use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::io;
use std::fmt::{Formatter,Display};
use std::error::Error;

#[derive(Debug)]
pub struct LeverFile {
	url: Option<String>,
	compile_commands: Vec<String>,
	install_commands: Vec<String>,
}

#[derive(Debug)]
pub enum LeverFileError {
	UnknownSection((String,usize)),
	IoError(std::io::Error),
}

impl Error for LeverFileError {}
impl Display for LeverFileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(),std::fmt::Error> {
		use self::LeverFileError::*;
		match self {
			UnknownSection((section,line)) => write!(f, "Unknown section {section} at line {line})"),
			IoError(error) => write!(f, "{error:?})"),
		}
	}
}

impl LeverFile {
	pub fn load<T: AsRef<Path>>(path: T) -> Result<Self,LeverFileError> {
		//defaults
		let mut url = None;
		let mut compile_commands = vec![];
		let mut install_commands = vec![];
		//load file
		let mut leverfile = String::new();
		let _ = match File::open(path.as_ref()){
			Ok(file) => file,
			Err(error) => return Err(LeverFileError::IoError(error))
		}.read_to_string(&mut leverfile);
		//====== read the leverfile line by line ======
		let mut section_name = String::from("global");
		for (line_numer,raw_line) in leverfile.split('\n').into_iter().enumerate() {
			let line = raw_line;
			//skip empty lines
			if line.len() == 0 {continue}
			//sections
			if line.chars().next() == Some('[') {
				section_name = line[1..].trim_end_matches(']').into();
				continue;
			}
			match section_name.as_str() {
				"url" => {
					url = Some(line.into())
				}
				"install" => {
					install_commands.push(line.into())
				}
				"compile" => {
					compile_commands.push(line.into())
				}
				_ => {
					return Err(LeverFileError::UnknownSection((section_name,line_numer+1)))
				}
			}
		}
		//pack struct
		Ok(Self {
			url,
			compile_commands,
			install_commands,
		})
	}
}
