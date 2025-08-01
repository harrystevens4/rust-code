mod parser {
	use std::collections::HashMap;
	//====== structures ======
	#[derive(Debug,PartialEq)]
	pub enum XMLParseError {
		TagNeverClosed(usize), //position
		EmptyTag,
	}
	#[derive(Debug,PartialEq)]
	pub struct ParsedXML {
		elements: Vec<XMLElement>,
	}
	#[derive(Debug,PartialEq)]
	pub struct XMLElement {
		name: String,
		attributes: Vec<HashMap<String,String>>,
	}
	#[derive(Debug,PartialEq)]
	pub enum XMLTagType {
		Start,
		End,
		Empty,
	}
	#[derive(Debug,PartialEq)]
	pub struct XMLTag {
		pub tag_type: XMLTagType,
		pub name: String,
		pub attributes: Vec<(String,String)>,
	}
	#[derive(Debug,PartialEq)]
	pub enum XMLItem {
		Tag(XMLTag), 
		Content(String),
	}
	//====== standalone functions ======
	fn parse_tag(raw_tag: String) -> Result<XMLTag,XMLParseError>{
		let items = raw_tag
			.as_str()
			.trim_start_matches("<")
			.trim_end_matches(">")
			.split(' ')
			.collect::<Vec<&str>>();
		let first_char = items.iter()
			.next()
			.ok_or(XMLParseError::EmptyTag)?
			.chars()
			.next()
			.ok_or(XMLParseError::EmptyTag)?;
		let last_char = items.iter()
			.last()
			.ok_or(XMLParseError::EmptyTag)?
			.chars()
			.last()
			.ok_or(XMLParseError::EmptyTag)?;
		Ok(XMLTag {
			//check the last char is '/' to confirm if it is an empty tag
			tag_type: match last_char {
				'/' => XMLTagType::Empty,
				_ => match first_char {
					'/' => XMLTagType::End,
					_ => XMLTagType::Start,
				}
			},
			name: items.iter().next().ok_or(XMLParseError::EmptyTag)?.trim_matches('/').to_string(),
			attributes: vec![],
		})
	}
	pub fn lexer(data: String) -> Result<Vec<XMLItem>,XMLParseError>{
		//split into vec of elements
		let mut lexemes = vec![];
		let mut data_iter = data.chars();
		let mut content = String::new();
		let mut position = 0_usize;
		loop {
			if let Some(ch) = data_iter.next() {
				//add data untill next '>' (tag)
				if ch == '<' {
					//add whatever element we have collected already
					if !(&mut content).is_empty() {lexemes.push(XMLItem::Content((&content).clone()))}
					(&mut content).clear();
					//find the ending '>'
					let mut tag = String::new();
					loop {
						let ch = data_iter.next().ok_or(XMLParseError::TagNeverClosed(position))?;
						position += 1;
						if ch == '<' {return Err(XMLParseError::TagNeverClosed(position))};
						if ch == '>' {
							lexemes.push(XMLItem::Tag(parse_tag(tag)?));
							break;
						}
						tag.push(ch);
					}
				}else {
					(&mut content).push(ch);
				}
				position += 1;
			}else {break}
		}
		Ok(lexemes)
	}
	//====== associated functions ======
	impl ParsedXML {

	}
	impl From<String> for ParsedXML {
		fn from(data: String) -> Self{
			ParsedXML {
				elements: vec![],
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::parser::XMLTagType::{Start,End,Empty};
	use super::parser::XMLItem::{Tag,Content};
	use super::parser::XMLTag;
	use super::parser::XMLParseError::*;

	#[test]
	fn lexer_ok(){
		let sample_xml = "<a>hello<br/></a>".to_string();
		let xml_tokens = parser::lexer(sample_xml).unwrap();
		assert_eq!(xml_tokens,vec![
			Tag(XMLTag {tag_type: Start, name: "a".to_string() ,attributes: vec![]}),
			Content("hello".to_string()),
			Tag(XMLTag {tag_type: Empty, name: "br".to_string() ,attributes: vec![]}),
			Tag(XMLTag {tag_type: End, name: "a".to_string() ,attributes: vec![]}),

		]);
	}
	#[test]
    fn lexer_err(){
		let sample_xml = "<ahello<br/></a>".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(TagNeverClosed(7)));
		let sample_xml = "<a>hello<br/</a>".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(TagNeverClosed(12)));
		let sample_xml = "<a>hello<br/></a".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(TagNeverClosed(15)));
	}
}
