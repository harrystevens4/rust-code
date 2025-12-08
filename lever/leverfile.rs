use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::io;

pub struct LeverFile {
	url: Option<String>,
	always_copy_files: Vec<(String,String)>,
	copy_once_files: Vec<(String,String)>,
	compile_command: String,
}

impl LeverFile {
	pub fn load<T: AsRef<Path>>(path: T) -> io::Result<Self> {
		//defaults
		let url = None;
		let always_copy_files = vec![];
		let copy_once_files = vec![];
		let compile_command = String::new();
		//load file

		let mut leverfile = String::new();
		let _ = File::open(path.as_ref())?.read_to_string(&mut leverfile);
		//pack struct
		Ok(Self {
			url,
			always_copy_files,
			copy_once_files,
			compile_command
		})
	}
}
