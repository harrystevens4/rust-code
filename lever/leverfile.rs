use std::path::Path;
use std::thread;
use std::process::{Stdio,Command};
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
	name: String,
}

#[derive(Debug)]
pub enum LeverFileError {
	UnknownSection((String,usize)),
	IoError(std::io::Error),
	MissingSection(String),
}

impl Error for LeverFileError {}
impl Display for LeverFileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(),std::fmt::Error> {
		use self::LeverFileError::*;
		match self {
			UnknownSection((section,line)) => write!(f, "Unknown section {section} at line {line})"),
			IoError(error) => write!(f, "{error:?})"),
			MissingSection(section) => write!(f, "Missing section {section:?})"),
		}
	}
}

impl LeverFile {
	pub fn load<T: AsRef<Path>>(path: T) -> Result<Self,LeverFileError> {
		//defaults
		let mut url = None;
		let mut name = String::new();
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
			//remove comments
			let line = if let Some(index) = raw_line.find('#') {
				raw_line[..index].to_string()
			}else {raw_line.to_string()};
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
				"name" => {
					name = line.into();
				}
				_ => {
					return Err(LeverFileError::UnknownSection((section_name,line_numer+1)))
				}
			}
		}
		//check for required fields
		if name.len() == 0 {
			return Err(LeverFileError::MissingSection("name".into()))
		}
		//pack struct
		Ok(Self {
			url,
			compile_commands,
			install_commands,
			name,
		})
	}
	pub fn compile<T: AsRef<Path>>(&self, path: T) -> io::Result<()>{
		println!("=== Compiling {} ===",self.name);
		for command in self.compile_commands.iter() {
			//start a shell to execute each line
			let mut child = Command::new("sh")
				.arg("-c")
				.arg(command)
				.stderr(Stdio::piped())
				.stdout(Stdio::piped())
				.stdin(Stdio::null())
				.spawn()?;
			println!("> {command}");
			//read and print stdout from shell
			if let Some(ref mut stdout) = child.stdout {
				let mut line_buffer = String::new();
				loop {
					//read some data
					let mut stdout_buffer = [0; 64];
					let count = stdout.read(&mut stdout_buffer)?;
					if count == 0 {break}
					//output in a nice format
					for byte in &stdout_buffer[..count] {
						let ch = char::from(*byte);
						if ch == '\n' {
							println!("==> {}",line_buffer);
							line_buffer.truncate(0);
						}else {
							line_buffer.push(ch);
						}
					}
				}
			}
			//once stdout closes print anything on stderr
			if let Some(ref mut stderr) = child.stderr {
				let mut line_buffer = String::new();
				loop {
					//read some data
					let mut stderr_buffer = [0; 64];
					let count = stderr.read(&mut stderr_buffer)?;
					if count == 0 {break}
					//output in a nice format
					for byte in &stderr_buffer[..count] {
						let ch = char::from(*byte);
						if ch == '\n' {
							println!("-e-> {}",line_buffer);
							line_buffer.truncate(0);
						}else {
							line_buffer.push(ch);
						}
					}
				}
			}
			let exit_status = child.wait()?;
			if !exit_status.success() {
				eprintln!("Process exited with an error");
				match exit_status.code() {
					Some(code) => println!("{command:?} exited with code {code}",),
					None => println!("{command:?} was terminated by signal",),
				}
				return Err(io::Error::other("Compilation error"))
			}
		}
		println!("Successfull compile");
		Ok(())
	}
}
