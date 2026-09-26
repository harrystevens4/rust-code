mod icalendar;
mod config;
mod tui;
use std::env;
use std::fs;
use std::io;
use std::{fmt::{Debug,Display,Formatter},fmt};
use icalendar::{CombinedCalendar,ICalendar};
use config::{ApplicationConfig,CalendarConfig};
use ratatui::style::Style;
use crate::tui::Application;
//use std::process::{ExitCode,ExitCode::FAILURE,ExitCode::SUCCESS};

const STYLE_SELECTED_TEXT: Style = Style::new().white().on_red();
const STYLE_HIGHLIGHTED_TEXT: Style = Style::new().underlined();
const DATE_FORMAT_STRING: &str = "%a - %d/%m/%Y";
const TIME_FORMAT_STRING: &str = "%H:%M";

//if io::Error could impl From<String> that would be incredible
#[macro_export]
macro_rules! fmt_err {
    ($($arg:tt)*) => {
        std::io::Error::other(std::fmt::format(format_args!($($arg)*)))
    }
}

//literalt just io::Error but the debug formatter is the default formatter
struct PlainError (io::Error);

impl From<io::Error> for PlainError {
    fn from(val: io::Error) -> PlainError {
        PlainError(val)
    }
}
impl Debug for PlainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error>{
        std::fmt::Display::fmt(&self.0,f)
    }
}
impl Display for PlainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error>{
        std::fmt::Display::fmt(&self.0,f)
    }
}

fn main() -> Result<(),PlainError> {
	//====== load config files if they exist ======
	let application_config = env::home_dir()
		.ok_or(io::Error::other("User's home directory not found"))
		.map(|d| d.join(".config/calendr/config.ini"))
		.map(|d| ApplicationConfig::load(d))
		.flatten()
		.inspect_err(|e| eprintln!("Error loading application config: {e}"))
		.unwrap_or_default(); //its not that deep if we cant load the config so just use the default one
	let calendar_config = env::home_dir()
		.ok_or(io::Error::other("User's home directory not found"))
		.map(|d| d.join(".config/calendr/calendars.ini"))
		.map(|d| CalendarConfig::load(d))
		.flatten()
		.inspect_err(|e| eprintln!("Error loading calendar config: {e}"))
		.unwrap_or_default();
	//====== fetch and load each calendar ======
	//pipeline from urls to calendar
	println!("loading calendars...");
	let calendar = calendar_config
        .calendars()
		.into_iter()
		.map(|e|{
            let name = e.name();
            if let Some(url) = e.url(){
                Ok((name,url))
            }else {
                Err(fmt_err!("no url for calendar {:?}",name))
            }
        })
		.collect::<Result<Vec<_>,_>>()
    	//check for any errors
        .map_err(|e| fmt_err!("Error reading calendars config file: {e}"))?
		.into_iter()
		//try loading from url
		.map(|(n,u)| match ICalendar::load_from_url(&n,&u){
			Ok(c) => Ok(c),
			//check for cache location
			Err(e1) => match application_config.calendar_storage_dir(){
				//if we cant load from url try from cache
				Some(d) => match fs::read_to_string(d.join(&n)){
					Ok(s) => {
						println!("Falling back on cached calendar {n:?} due to error: {e1}");
						ICalendar::load_from_str(n,s)
					},
					//if we cant load from cache its an epic fail
					Err(e2) => Err(fmt_err!("Error loading {n:?} from url: {e1}\n followed by error loading from cache: {e2}")),
				},
				None => Err(fmt_err!("Error loading {n:?} from url: {e1}\nNo fallback cache dir available"))
			}
		})
		.collect::<Result<CombinedCalendar,_>>()
		.map_err(|e| fmt_err!("Error loading calendar: {e}"))?;
	//====== cache the calendars ======
	if let Some(calendar_dir) = application_config.calendar_storage_dir(){
		println!("caching downloads...");
		fs::create_dir_all(&calendar_dir)
			.map_err(|e| fmt_err!("mkdir({calendar_dir:?}): {e}"))?;
		calendar
			.calendars()
			.into_iter()
			.map(|c| fs::write(calendar_dir.join(c.name()),c.as_ics())
				.map_err(|e| fmt_err!("Error caching calendars: {e}"))
			)
			.collect::<Result<Vec<_>,_>>()?;
	}
	//====== ratatui ======
	let mut application = Application::new(calendar);
	ratatui::run(move |terminal| application.tui_loop(terminal))
        .map_err(|e| fmt_err!("Error in tui loop: {e}"))?;
    Ok(())
}
