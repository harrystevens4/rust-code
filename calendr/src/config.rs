use std::path::Path;
use std::fs;
use std::io;
use iniconfig::{ConfigFile,ConfigSection};
use std::default::Default;

#[derive(Debug,Clone)]
pub struct CalendarInfo {
    name: String,
    url: Option<String>,
}

#[derive(Debug,Clone)]
pub struct CalendarConfig {
    calendars: Vec<CalendarInfo>,
}

#[derive(Debug,Clone)]
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
            calendars: vec![],
		}
	}
}

impl CalendarConfig {
	pub fn load(path: impl AsRef<Path>) -> io::Result<CalendarConfig> {
        //read the config file
		let config_contents = fs::read_to_string(path)?;
		let config = ConfigFile::from(config_contents.as_str());
        //grab the section name and url
        let calendars = config
            .into_iter()
            .map(|section| CalendarInfo {
                name: section.name().to_string(),
                url: section
                    .properties()
                    .get("url")
                    .map(String::from)
                    .map(|s| if s.is_empty() {None} else {Some(s)})
                    .flatten(),
            })
            .collect::<Vec<_>>();
        //build our calendar config
        Ok(CalendarConfig {
            calendars
        })

	}
	pub fn calendars(&self) -> Vec<CalendarInfo>{
		self.calendars.clone()
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

impl CalendarInfo {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn url(&self) -> Option<String> {
        self.url.clone()
    }
}
