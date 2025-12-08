use std::path::Path;

pub struct LeverFile {
	url: Option<String>,
	always_copy_files: Vec<(String,String)>,
	copy_once_files: Vec<(String,String)>,
	compile_command: String,
}

impl LeverFile {
	fn load(path: &Path) -> Self {
		//defaults
		let url = None;
		let always_copy_files = vec![];
		let copy_once_files = vec![];
		let compile_command = String::new()
		//load file
		//pack struct
		Self {
			url,
			always_copy_files,
			copy_once_files,
			compile_command
		}
	}
}
