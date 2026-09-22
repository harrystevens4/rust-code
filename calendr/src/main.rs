mod icalendar;
mod config;
mod tui;
use std::env;
use std::io;
use std::path::{PathBuf,Path};
use std::error::Error;
use icalendar::CombinedCalendar;
use config::{ApplicationConfig,CalendarConfig};
use ratatui::style::Style;
use crate::tui::Application;

const STYLE_SELECTED_TEXT: Style = Style::new().white().on_red();
const STYLE_HIGHLIGHTED_TEXT: Style = Style::new().underlined();
const DATE_FORMAT_STRING: &str = "%a - %d/%m/%Y";
const TIME_FORMAT_STRING: &str = "%H:%M";

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

fn main() -> Result<(),()>{
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
    dbg!{&calendar_config};
	//====== fetch and load each calendar ======
	let calendar_urls = calendar_config
        .calendars()
		.into_iter()
		.map(|e| (e.name(),e.url()))
		.collect();
	let calendar = match CombinedCalendar::load_from_urls(calendar_urls){
		Ok(c) => c,
		Err(e) => {
			eprintln!("Error loading calendars: {e}");
			return Err(());
		}
	};
	//====== ratatui ======
	let mut application = Application::new(calendar);
	if let Err(e) = ratatui::run(move |terminal| application.tui_loop(terminal)){
		eprintln!("Error in tui loop: {e}");
		return Err(());
	}
	Ok(())
}
