/* ~/.config/calendr/calendars.ini

[calendar a]
url=https://link-to-ical.ics
[calendar b]
url=https://example

*/
/* ~/.config/calendr/config.ini

[storage]
cache_web_calendars=true
calendar_dir=/home/john/.local/share/calendr/

*/

use std::path::Path;
use std::fs;
use std::io;
use std::env;
use std::path::PathBuf;
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
	cache_web_calendars: bool,
	calendar_storage_dir: Option<PathBuf>,
}

impl Default for ApplicationConfig {
	fn default() -> ApplicationConfig {
		ApplicationConfig {
			cache_web_calendars: true,
			calendar_storage_dir: env::home_dir()
				.map(|h| h.join(".local/share/calendr"))
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
		let mut application_config = Self::default();
		for section in config {
            match section.name(){
                "storage" => {
					if let Some(calendar_dir) = section
						.properties()
						.get("calendar_dir")
						.map(PathBuf::from)
					{
						application_config.calendar_storage_dir = Some(calendar_dir);
					}
					if let Some(cache_status) = section
						.properties()
						.get("cache_web_calendars")
						.map(|s| if s.to_ascii_lowercase() == "true" {true} else {false})
					{
						application_config.cache_web_calendars = cache_status
					}
				},
				_ => ()
            }
		}
		Ok(application_config)
	}
	pub fn calendar_storage_dir(&self) -> Option<PathBuf> {
		self.calendar_storage_dir.clone()
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
