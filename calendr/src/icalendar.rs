use std::io;
use std::iter::Peekable;
use std::collections::HashMap;
use std::time::{SystemTime,Duration};
use chrono::{DateTime,Utc,Datelike,Local};
use uuid::Uuid;

#[derive(Debug,PartialEq)]
pub enum ICComponentType {
	Event,
	Alarm,
	Todo,
	Journal,
	FreeBusy,
	TimeZone,
	Calendar,
	TimeZoneStandard,
	TimeZoneDaylight,
	Other,
}

#[derive(Debug)]
struct ICComponent {
	component_type: ICComponentType,
	properties: HashMap<String,String>,
	children: Vec<ICComponent>,
}

//slightly higher level abstration of ICComponent
#[derive(Debug,Clone)]
pub struct CalendarEvent {
	parent_calendar_name: String,
	title: String,
	start_time: DateTime<Local>,
	end_time: DateTime<Local>,
	uuid: String,
	description: String,
}

//allows merging of multiple ICalendar's
#[derive(Debug)]
pub struct CombinedCalendar {
	calendars: Vec<ICalendar>,
	events: HashMap<usize,Vec<CalendarEvent>>, //usize is a date here like 15092026
}

#[derive(Debug)]
pub struct ICalendar {
	root_component: ICComponent, //like VCALENDAR for a full calendar or VEVENT for a single event
	name: String,
}

impl From<&str> for ICComponentType {
	fn from(string: &str) -> ICComponentType {
		use ICComponentType::*;
		match string {
			"VEVENT" => Event,
			"VALARM" => Alarm,
			"VTODO" => Todo,
			"VJOURNAL" => Journal,
			"VFREEBUSY" => FreeBusy,
			"VTIMEZONE" => TimeZone,
			"VCALENDAR" => Calendar,
			"STANDARD" => TimeZoneStandard,
			"DAYLIGHT" => TimeZoneDaylight,
			_ => Other,
		}
	}
}

fn iso_to_local_time(iso_time: &str) -> DateTime<Local> {
	DateTime::parse_from_rfc3339(iso_time)
		.unwrap_or(DateTime::<Utc>::from(SystemTime::now()).into())
		.into()
}
fn get_date_hash(date: impl Datelike) -> usize {
	let year = date.year() as usize;
	let month = date.month() as usize;
	let day = date.day() as usize;
	day + month*100 + year*10000
}

impl ICalendar {
	pub fn load_from_str(name: impl AsRef<str>, data: impl AsRef<str>) -> io::Result<ICalendar> {
		let data = data.as_ref();
		//remove trailing CRLF
		let data = data.trim_end_matches("\r\n");
		//unfold continuations
		let data = data.replace("\t"," ").replace("\r\n ","");
		//split into lines
		let mut lines = data.split("\r\n").peekable();
		//parse
		let root_component = ICalendar::parse(&mut lines)?;
		//return ICalendar
		Ok(ICalendar {
			root_component,
			name: name.as_ref().into(),
		})
	}
	fn parse<'a, I: Iterator<Item = &'a str>>(lines: &mut Peekable<I>) -> io::Result<ICComponent> {
		//====== prepare our component ======
		let mut component = ICComponent {
			children: vec![],
			component_type: ICComponentType::Other,
			properties: HashMap::new(),
		};
		let mut component_type: Option<ICComponentType> = None;
		//====== parse ======
		loop {
			let Some(line) = lines.peek() else {break};
			//====== process line ======
			let Some((name_and_params,value)) = line.split_once(":")
			else {
				return Err(io::Error::other(format!("Line missing colon: {line:?}")));
			};
			let mut name_and_params_iter = name_and_params.split(";");
			let Some(name) = name_and_params_iter.next()
			else {
				return Err(io::Error::other(format!("split(\";\") yielded no values: {line:?}")));
			};
			let params: HashMap<_,_> = name_and_params_iter
				.filter_map(|p| p.split_once("="))
				.collect();
			//====== handle different tags ======
			if name == "BEGIN" {
				if component_type.is_none(){
					component_type = Some(value.into());
				}else {
					component.children.push(ICalendar::parse(lines)?);
				}
			}else if name == "END" {
				if component_type != Some(ICComponentType::from(value)){
					return Err(io::Error::other(format!("BEGIN {component_type:?} does not match {line}")));
				}
				if let Some(component_type) = component_type {
					component.component_type = component_type;
				}
				return Ok(component);
			}else {
				component.properties.insert(name.into(),value.into());
				for (key,value) in params {
					component.properties.insert(name.to_string()+"_"+key,value.to_string());
				}
			}
			//consume line
			let _ = lines.next();
		}
		todo!();
	}
	pub fn name(&self) -> String {
		self.name.clone()
	}
	pub fn get_calendar_events(&self) -> Vec<CalendarEvent>{
		let mut events = vec![];
		let mut ic_event_components = vec![];
		//====== just one event or a whole calendar? ======
		if self.root_component.component_type == ICComponentType::Event {
			ic_event_components.push(&self.root_component);
		}else if self.root_component.component_type == ICComponentType::Calendar {
			let mut event_children: Vec<_> = self.root_component.children
				.iter()
				.filter(|c| c.component_type == ICComponentType::Event)
				.collect();
			ic_event_components.append(&mut event_children);
		}
		//====== process each of those events ======
		for ic_event_component in ic_event_components {
			let properties = &ic_event_component.properties;
			events.push(CalendarEvent::from_calendar(self)
				.with_title(properties.get("SUMMARY").unwrap_or(&String::new()))
				.with_description(properties.get("DESCRIPTION").unwrap_or(&String::new()))
				.with_start_time(properties
					.get("DTSTART")
					.map(|t| iso_to_local_time(t))
					.unwrap_or(Local::now())
				)
				.with_end_time(properties
					.get("DTEND")
					.map(|t| iso_to_local_time(t))
					.unwrap_or(Local::now())
				)
				.with_uuid(&properties
					.get("UID")
					.map(String::from)
					.unwrap_or(Uuid::new_v4().simple().to_string())
				)
			)
		}
		events
	}

}

impl CombinedCalendar {
	pub fn load_from_strings(strings: Vec<(&str,&str)>) -> io::Result<CombinedCalendar>{
		//====== prepare the combined calendar ======
		let mut combined_calendar = CombinedCalendar {
			calendars: Vec::new(),
			events: HashMap::new(),
		};
		//====== load each calendar ======
		for (name,raw_icalendar) in strings {
			let calendar = ICalendar::load_from_str(name,raw_icalendar)?;
			let events = calendar.get_calendar_events();
			combined_calendar.calendars.push(calendar);
			for event in events {
				let hash = event.get_date_hash();
				let Some(ref mut date_event_list) = (if combined_calendar.events.contains_key(&hash) {
					combined_calendar.events.get_mut(&hash)
				}else {
					combined_calendar.events.insert(hash,vec![]);
					combined_calendar.events.get_mut(&hash)
				})
				else {continue};
				date_event_list.push(event);
			}
		}
		println!("{:#?}",combined_calendar.events);
		Ok(combined_calendar)
	}
	pub fn get_events_for_date(&self,date: impl Datelike) -> Vec<CalendarEvent>{
		//====== pull out events for the selected day ======
		let Some(events) = self.events.get(&get_date_hash(date))
		else {return vec![]};
		events.to_vec()
	}
}

impl CalendarEvent {
	pub fn from_calendar(calendar: &ICalendar) -> CalendarEvent {
		CalendarEvent {
			parent_calendar_name: calendar.name(),
			title: String::new(),
			description: String::new(),
			start_time: Local::now(),
			end_time: Local::now(),
			uuid: Uuid::new_v4().simple().to_string(),
		}.with_duration(Duration::from_hours(1))
	}
	pub fn get_date_hash(&self) -> usize {
		get_date_hash(self.start_time)
	}
	pub fn with_start_time(mut self, time: DateTime<Local>) -> CalendarEvent {
		self.start_time = time;
		self
	}
	pub fn with_end_time(mut self, time: DateTime<Local>) -> CalendarEvent {
		self.end_time = time;
		self
	}
	//essentialy add_end_time but does the maths for you
	pub fn with_duration(mut self, duration: Duration) -> CalendarEvent {
		self.end_time = self.start_time + duration;
		self
	}
	pub fn with_title(mut self, title: &str) -> CalendarEvent {
		self.title = title.to_string();
		self
	}
	pub fn with_uuid(mut self, uuid: &str) -> CalendarEvent {
		self.uuid = uuid.to_string();
		self
	}
	pub fn with_description(mut self, description: &str) -> CalendarEvent {
		self.description = description.to_string();
		self
	}
}
