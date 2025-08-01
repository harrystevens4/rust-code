mod parser {
	use std::collections::HashMap;
	//====== structures ======
	//provided as the interface to the user
	#[derive(Debug,PartialEq)]
	pub enum XMLParseError {
		TagNeverClosed(usize), //position
		EmptyTag(usize),
		MalformedAttriubute(usize),
	}
	#[derive(Debug,PartialEq)]
	pub struct XMLTree {
		elements: Vec<XMLElement>,
	}
	#[derive(Debug,PartialEq)]
	pub struct XMLElement {
		name: String,
		attributes: Vec<HashMap<String,String>>,
		content: Option<String>,
		elements: Vec<XMLElement>,
	}
	//used by the lexer
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
		pub attributes: HashMap<String,String>,
	}
	#[derive(Debug,PartialEq)]
	pub enum XMLItem {
		Tag(XMLTag), 
		Content(String),
	}
	//====== standalone functions ======
	//                         purely for the errors
	fn parse_attributes(attribute_list: Vec<String>, overall_position: usize) -> Result<HashMap<String,String>,XMLParseError>{
		let mut attributes = HashMap::new();
		let mut current_position = 0;
		for attr in attribute_list
			.into_iter()
			.map(|s| s.trim_matches('/').to_string())
		{
				if !attr.is_empty(){
					//only a=b not a or a=b=c
					let attr_split = attr.split('=').map(|s| s.to_string()).collect::<Vec<String>>();
					if attr_split.len() != 2 {return Err(XMLParseError::MalformedAttriubute(current_position))}
					//seperate and unquote
					let value = attr_split[1].trim_matches('\"');
					attributes.insert(attr_split[0].to_string(), value.to_string());
				}
				current_position += attr.len()+1;
		}
		Ok(attributes)
	}
	fn parse_tag(raw_tag: String, position: usize) -> Result<XMLTag,XMLParseError>{
		//clean it up and split it by spaces
		let items = raw_tag
			.as_str()
			.trim_start_matches("<")
			.trim_end_matches(">")
			.split(' ')
			.collect::<Vec<&str>>();
		//get first char
		let first_char = items.iter()
			.filter(|x| !x.is_empty()) //remove blanks
			.next()
			.ok_or(XMLParseError::EmptyTag(position))?
			.chars()
			.next()
			.ok_or(XMLParseError::EmptyTag(position))?;
		//get last char
		let last_char = items.iter()
			.filter(|x| !x.is_empty()) //remove blanks
			.last()
			.ok_or(XMLParseError::EmptyTag(position))?
			.chars()
			.last()
			.ok_or(XMLParseError::EmptyTag(position))?;
		//check the tag isnt empty: "<>" or "</>"
		if items[0].len() == 1 && items[0].chars().next() == Some('/') {
			return Err(XMLParseError::EmptyTag(position))
		}
		//construct the tag
		Ok(XMLTag {
			//check the last char is '/' to confirm if it is an empty tag
			tag_type: match last_char {
				'/' => XMLTagType::Empty,
				_ => match first_char {
					'/' => XMLTagType::End,
					_ => XMLTagType::Start,
				}
			},
			name: items.iter().next().ok_or(XMLParseError::EmptyTag(position))?.trim_matches('/').to_string(),
			attributes: parse_attributes(
				items.into_iter().map(|s| s.to_string()).collect::<Vec<String>>()[1..].into(),position
			)?,
		})
	}
	pub fn lexer(data: String) -> Result<Vec<XMLItem>,XMLParseError>{
		//split into vec of elements
		let mut lexemes = vec![];
		let mut data_iter = data.chars().peekable();
		let mut content = String::new();
		let mut position = 0_usize;
		loop {
			if let Some(ch) = data_iter.next() {
				//add data untill next '>' (tag)
				if ch == '<' {
					//====== process comments ======
					if *data_iter.peek().ok_or(XMLParseError::TagNeverClosed(position))? == '!' {
						let start_position = position + 1;
						//consume the comment
						let mut buffer = vec![];
						//grab 3 to start off with
						for _i in 0..3 {
							buffer.push(data_iter.next().ok_or(XMLParseError::TagNeverClosed(start_position))?);
							position += 1;
						}
						//keep looking one by one
						loop{
							buffer.push(data_iter.next().ok_or(XMLParseError::TagNeverClosed(start_position))?);
							buffer.remove(0);
							position += 1;
							if buffer.iter().collect::<String>() == "-->" {break}
						}
					//====== all other tags ======
					}else {
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
								lexemes.push(XMLItem::Tag(parse_tag(tag,position)?));
								break;
							}
							tag.push(ch);
						}
					}
				}else {
					(&mut content).push(ch);
				}
				position += 1;
			}else {break}
		}
		Ok(lexemes)
	}
	pub fn build_from_lexer(lexemes: Vec<XMLItem>) -> Result<Vec<XMLElement>,XMLParseError>{
		todo!()
	}
	//====== associated functions ======
	impl TryFrom<String> for XMLTree {
		type Error = XMLParseError; 
		fn try_from(data: String) -> Result<Self,XMLParseError> {
			Ok(XMLTree {
				elements: vec![],
			})
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;
	use super::parser::XMLTagType::{Start,End,Empty};
	use super::parser::XMLItem::{Tag,Content};
	use super::parser::XMLTag;
	use super::parser::XMLParseError::*;

	#[test]
	fn lexer_ok(){
		let sample_xml = "<a>hello<br/></a>".to_string();
		let xml_tokens = parser::lexer(sample_xml).unwrap();
		assert_eq!(xml_tokens,vec![
			Tag(XMLTag {tag_type: Start, name: "a".to_string() ,attributes: HashMap::new()}),
			Content("hello".to_string()),
			Tag(XMLTag {tag_type: Empty, name: "br".to_string() ,attributes: HashMap::new()}),
			Tag(XMLTag {tag_type: End, name: "a".to_string() ,attributes: HashMap::new()}),

		]);
		let sample_xml = "<a volume=\"50\" dial=\"max\">hello<br/></a>".to_string();
		let xml_tokens = parser::lexer(sample_xml).unwrap();
		assert_eq!(xml_tokens,vec![
			Tag(XMLTag {tag_type: Start, name: "a".to_string() ,attributes: HashMap::from([("volume".to_string(),"50".to_string()),("dial".into(),"max".into())])}),
			Content("hello".to_string()),
			Tag(XMLTag {tag_type: Empty, name: "br".to_string() ,attributes: HashMap::new()}),
			Tag(XMLTag {tag_type: End, name: "a".to_string() ,attributes: HashMap::new()}),

		]);
	}
	#[test]
    fn lexer_err_tag_never_closed(){
		let sample_xml = "<ahello<br/></a>".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(TagNeverClosed(7)));
		let sample_xml = "<a>hello<br/</a>".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(TagNeverClosed(12)));
		let sample_xml = "<a>hello<br/></a".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(TagNeverClosed(15)));
	}
	#[test]
	fn lexer_err_empty_tag(){
		let sample_xml = "<a></>world</a>".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(EmptyTag(5)));
		let sample_xml = "<a><>world</a>".to_string();
		assert_eq!(parser::lexer(sample_xml),Err(EmptyTag(4)));
	}
	#[test]
	fn lexer_comment(){
		let sample_xml = "<a>an<!-- hiiii <no need to escape> -->gry</a>".to_string();
		assert_eq!(parser::lexer(sample_xml),Ok(vec![
			Tag(XMLTag {tag_type: Start, name: "a".to_string() ,attributes: HashMap::new()}),
			Content("angry".to_string()),
			Tag(XMLTag {tag_type: End, name: "a".to_string() ,attributes: HashMap::new()}),
		]));

	}
	#[test]
	fn lexer_with_spaces(){
	}
}
