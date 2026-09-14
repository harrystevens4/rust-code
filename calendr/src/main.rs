mod icalendar;
use std::env;
use std::io;
use std::error::Error;
use icalendar::ICalendar;

fn main(){
	let Some(calendar_url) = env::args().nth(1)
	else {
		eprintln!("please provide ics url as first argument");
		return;
	};
	let calendar_raw = match reqwest::blocking::get(calendar_url).map(|c| c.text()).flatten(){
		Ok(c) => c,
		Err(e) => {
			eprintln!("Error fetching calendar: {e}");
			return;
		}
	};
	let calendar = ICalendar::load_from_str("calendar1",calendar_raw);
	println!("{calendar:#?}");
}
