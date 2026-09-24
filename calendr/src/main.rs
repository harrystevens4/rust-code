mod icalendar;
mod config;
mod tui;
use std::env;
use std::io;
use std::{fmt::{Debug,Display,Formatter},fmt};
use std::path::{PathBuf,Path};
use std::error::Error;
use icalendar::CombinedCalendar;
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

pub trait PathConcat {
	//why doesnt pathbuf impl Add<&Path> ????
	fn concat<T>(self,other: T) -> PathBuf
	where Self: AsRef<Path> + Sized, T: AsRef<Path> {
		let mut new_path = PathBuf::new();
		new_path.push(self);
		new_path.push(other);
		new_path
	}
}

impl PathConcat for Path {}
impl PathConcat for PathBuf {}

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
		.map(|d| d.concat(".config/calendr/config.ini"))
		.map(|d| ApplicationConfig::load(d))
		.flatten()
		.inspect_err(|e| eprintln!("Error loading application config: {e}"))
		.unwrap_or_default(); //its not that deep if we cant load the config so just use the default one
	let calendar_config = env::home_dir()
		.ok_or(io::Error::other("User's home directory not found"))
		.map(|d| d.concat(".config/calendr/calendars.ini"))
		.map(|d| CalendarConfig::load(d))
		.flatten()
		.inspect_err(|e| eprintln!("Error loading calendar config: {e}"))
		.unwrap_or_default();
	//====== fetch and load each calendar ======
	let calendar_urls = calendar_config
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
		.collect::<Result<Vec<_>,_>>();
    //check for any errors
    let calendar_urls = calendar_urls
        .map_err(|e| fmt_err!("Error reading calendars config file: {e}"))?;
	let calendar = CombinedCalendar::load_from_urls(calendar_urls)
        .map_err(|e| fmt_err!("Error loading calendars: {e}"))?;
	//====== ratatui ======
	let mut application = Application::new(calendar);
	ratatui::run(move |terminal| application.tui_loop(terminal))
        .map_err(|e| fmt_err!("Error in tui loop: {e}"))?;
    Ok(())
}
