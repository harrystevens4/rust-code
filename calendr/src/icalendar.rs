use std::io;
use std::iter::Peekable;
use std::collections::HashMap;

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
}
