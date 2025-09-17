#![feature(trim_prefix_suffix)]
#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug,Clone)]
pub struct ConfigFile {
	sections: Vec<ConfigSection>,
}
#[derive(Debug,Clone)]
pub struct ConfigSection {
	name: String,
	properties: HashMap<String,String>,
}

impl ConfigSection {
	pub fn name<'a>(&'a self) -> &'a str {
		&self.name
	}
	pub fn properties<'a>(&'a self) -> &'a HashMap<String,String> {
		&self.properties
	}
}
impl ConfigFile {
	fn new() -> Self {
		Self { sections: vec![] }
	}
}
impl IntoIterator for ConfigFile {
	type Item = ConfigSection;
	type IntoIter = <Vec<ConfigSection> as IntoIterator>::IntoIter;
	fn into_iter(self) -> Self::IntoIter {
		self.sections.clone().into_iter()
	}
}
impl From<&str> for ConfigFile {
	fn from(data: &str) -> Self {
		Self {
			sections: parse_config(data),
		}
	}
}

fn parse_config(raw_config: &str) -> Vec<ConfigSection>{
	let lines = raw_config.split('\n').collect::<Vec<&str>>();
	let mut sections = read_label("top_level",lines.into_iter());
	//the first "section" will be everything before the first label, so strip it off
	sections.remove(0);
	sections
}
fn is_label(line: &str) -> bool {
	if line.trim().chars().nth(0) == Some('[') {return true}
	false
}
fn extract_label(line: &str) -> String {
	line.trim().trim_prefix("[").trim_suffix("]").into()
}
fn read_label<'a>(name: &str, mut lines: impl Iterator<Item = &'a str>) -> Vec<ConfigSection>{
	let mut section = ConfigSection {
		properties: HashMap::new(),
		name: name.into(),
	};
	//====== for line in lines ======
	loop {
		//the flipping for loop calls into_iterator which moves. SO ANOYING
		let line = match lines.next() { Some(v) => crop_comments(v), None => break, };
		//====== if it is a valid property, add it to the hashmap ======
		if is_property(line){
			let (key, value) = extract_property(line);
			let _ = section.properties.insert(key,value);
		}
		//====== recurse if another label is reached ======
		if is_label(line){
			let section_name = extract_label(line);
			let mut result = vec![section];
			result.append(&mut read_label(&section_name,lines));
			return result
		}
	}
	//base case
	vec![section]
}
fn is_property(line: &str) -> bool {
	line.contains('=')
}
fn extract_property(line: &str) -> (String,String){
	let result = line.split('=').take(2).collect::<Vec<_>>();
	(result[0].trim().into(), result[1].trim().into())
}
fn crop_comments<'a>(line: &'a str) -> &'a str {
	line.split('#').next().unwrap()
}
