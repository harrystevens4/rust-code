mod icalendar;
mod tui;
use std::env;
use std::io;
use std::error::Error;
use icalendar::CombinedCalendar;
use ratatui::style::Style;
use crate::tui::Application;

const STYLE_SELECTED_TEXT: Style = Style::new().on_red();
const STYLE_HIGHLIGHTED_TEXT: Style = Style::new().underlined();
const DATE_FORMAT_STRING: &str = "%a - %d/%m/%Y";
const TIME_FORMAT_STRING: &str = "%H:%M";

fn main() -> Result<(),()>{
	//====== grab the calendars ======
	let calendar_urls: Vec<_> = env::args()
		.skip(1)
		.collect();
	if calendar_urls.len() == 0 {
		eprintln!("please provide ics urls as command line arguments");
		return Err(());
	}
	//====== fetch and load each one ======
	let calendar_urls = calendar_urls
		.into_iter()
		.enumerate()
		.map(|(n,url)| (format!("Url calendar {}",n),url))
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
