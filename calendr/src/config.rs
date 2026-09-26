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

[ui]
date_format_string="%a - %d/%m/%Y"
time_format_string="%H:%M"
default_view_size=7
*/

use std::path::Path;
use std::fs;
use std::io;
use std::env;
use std::path::PathBuf;
use iniconfig::{ConfigFile};
use std::default::Default;
use crate::fmt_err;

const DATE_FORMAT_STRING: &str = "%a - %d/%m/%Y";
const TIME_FORMAT_STRING: &str = "%H:%M";

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
	date_format_string: String,
	time_format_string: String,
	default_view_size: usize,
}

impl Default for ApplicationConfig {
	fn default() -> ApplicationConfig {
		ApplicationConfig {
			cache_web_calendars: true,
			calendar_storage_dir: env::home_dir()
				.map(|h| h.join(".local/share/calendr")),
			date_format_string: DATE_FORMAT_STRING.to_string(),
			time_format_string: TIME_FORMAT_STRING.to_string(),
			default_view_size: 3,
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
		let config_file = ConfigFile::from(config_contents.as_str());
		let mut config = Self::default();
		for section in config_file {
            match section.name(){
				//====== ui section ======
                "ui" => {
					for (key,value) in section.properties() { match key.as_str() {
						"date_format_string" => config.date_format_string = value.into(),
						"time_format_string" => config.time_format_string = value.into(),
						"default_view_size" => config.default_view_size = value
							.parse()
							.map_err(|e| fmt_err!("Error parsing default_view_size: {e}"))?,
						_ => Err(fmt_err!("Unknown key {:?} in section {:?}",key,section.name()))?
					}}
				},
				//====== storage section ======
                "storage" => {
					for (key,value) in section.properties() { match key.as_str() {
						"calendar_dir" => config.calendar_storage_dir = Some(value.into()),
						"cache_web_calendars" => config.cache_web_calendars = match value.as_str() {
							"True" | "true" => true,
							_ => false,
						},
						_ => Err(fmt_err!("Unknown key {:?} in section {:?}",key,section.name()))?
					}}
				},
				//====== unknown section ======
				_ => Err(fmt_err!("Unknown section {:?}",section.name()))?
            }
		}
		Ok(config)
	}
	pub fn calendar_storage_dir(&self) -> Option<PathBuf> {
		self.calendar_storage_dir.clone()
	}
	pub fn date_format(&self) -> String {
		self.date_format_string.clone()
	}
	pub fn time_format(&self) -> String {
		self.time_format_string.clone()
	}
	pub fn default_view_size(&self) -> usize {
		self.default_view_size
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
