use std::path::Path;
use std::fs;
use std::io;
use iniconfig::{ConfigFile,ConfigSection};
use std::default::Default;

pub struct CalendarConfig {
}

pub struct ApplicationConfig {
}

impl Default for ApplicationConfig {
	fn default() -> ApplicationConfig {
		ApplicationConfig {
		}
	}
}
impl Default for CalendarConfig {
	fn default() -> CalendarConfig {
		CalendarConfig {
		}
	}
}

impl CalendarConfig {
	pub fn load(path: impl AsRef<Path>) -> io::Result<CalendarConfig> {
		Ok(Self::default())
	}
	pub fn calendars(&self) -> Vec<(String,String)>{
		vec![]
	}
}

impl ApplicationConfig {
	pub fn load(path: impl AsRef<Path>) -> io::Result<ApplicationConfig> {
		let config_contents = fs::read_to_string(path)?;
		let config = ConfigFile::from(config_contents.as_str());
		for section in config {
		}
		Ok(Self::default().into())
	}
}
