mod parser {
	struct UntilIter<'a,I,T: Iterator<Item = I>, F: FnMut(&I) -> bool> {
		test: F,
		iter: &'a mut T,
	}
	impl<'a,I,T: Iterator<Item = I>, F: FnMut(&I) -> bool> Iterator for UntilIter<'a,I,T,F> {
		type Item = I;
		fn next(&mut self) -> Option<I> {
			let item = self.iter.next()?;
			if (self.test)(&item) == true {return None}
			Some(item)
		}
	}
	fn until_matches<I,T: Clone + Iterator<Item = I>, F: FnMut(&I) -> bool>(iter: &mut T,mut test: F) -> Option<UntilIter<I,T,F>>{
		let _ = iter.clone().find(&mut test)?;
		Some(UntilIter {test,iter})
	}
	
	use std::collections::HashMap;
	//====== structures ======
	//provided as the interface to the user
	#[derive(Debug,PartialEq,Clone)]
	pub enum XMLParseError {
		TagNeverClosed(usize), //position
		EmptyTag(usize),
		MalformedAttriubute(usize),
		NoEndTag(String), //the index into all the tags
		UnexpectedEndTag(String),
		UnexpectedContent(String),
	}
	#[derive(Debug,PartialEq,Clone)]
	pub struct XMLTree {
		pub elements: Vec<XMLElement>,
	}
	#[derive(Debug,PartialEq,Clone)]
	pub struct XMLElement {
		name: String,
		attributes: HashMap<String,String>,
		content: Option<String>,
		elements: Vec<XMLElement>,
	}
	//used by the lexer
	#[derive(Debug,PartialEq,Clone)]
	pub enum XMLTagType {
		Start,
		End,
		Empty,
	}
	#[derive(Debug,PartialEq,Clone)]
	pub struct XMLTag {
		pub tag_type: XMLTagType,
		pub name: String,
		pub attributes: HashMap<String,String>,
	}
	#[derive(Debug,PartialEq,Clone)]
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
					if ![' ','\n','\t'].contains(&ch) {(&mut content).push(ch)}
				}
				position += 1;
			}else {break}
		}
		Ok(lexemes)
	}
	pub fn build_from_tokens(lexemes: Vec<XMLItem>) -> Result<Vec<XMLElement>,XMLParseError>{
		use XMLTagType::*;
		let mut array: Vec<XMLElement> = vec![];
		let mut lexemes_iter = lexemes.iter();
		loop {
			if let Some(token_type) = lexemes_iter.next(){
				array.push(match token_type {
					XMLItem::Tag(current_token) => match &current_token.tag_type {
						Start => { //recursion
							//grab untill end tag
							let mut contains = until_matches(&mut lexemes_iter,|x|{if let XMLItem::Tag(tag) = x && tag.name == current_token.name {true} else {false}})
								.ok_or(XMLParseError::NoEndTag(current_token.name.clone()))?
								.map(|t| t.clone())
								.collect::<Vec<_>>();
							XMLElement { 
								name: current_token.name.clone(),
								content: { 
									if contains.len() > 0 && let XMLItem::Content(c) = &contains[0] {Some(contains.remove(0).unwrap())} else {None}
								},
								elements: build_from_tokens(contains)?,
								attributes: current_token.attributes.clone(),
							}
						},
						End => return Err(XMLParseError::UnexpectedEndTag(current_token.name.clone())),
						Empty => XMLElement { name: current_token.name.clone(), attributes: current_token.attributes.clone(), content: None, elements: vec![] },
					},
					XMLItem::Content(c) => return Err(XMLParseError::UnexpectedContent(c.to_string())),
				})
			}else {break Ok(array)}
		}
		
	}
	//====== associated functions ======
	impl TryFrom<String> for XMLTree {
		type Error = XMLParseError; 
		fn try_from(data: String) -> Result<Self,XMLParseError> {
			Ok(XMLTree {
				elements: build_from_tokens(lexer(data)?)?,
			})
		}
	}
	impl TryFrom<XMLItem> for String {
		type Error = ();
		fn try_from(item: XMLItem) -> Result<Self,Self::Error> {
			match item {
				XMLItem::Tag(_) => Err(()),
				XMLItem::Content(c) => Ok(c),
			}
		}
	}
	impl XMLElement {
		pub fn builder(name: &str) -> Self {
			Self {
				name: name.to_string(),
				attributes: HashMap::new(),
				content: None,
				elements: vec![],
			}
		}
		pub fn attributes<const N: usize>(mut self, attributes: [(&str,&str); N]) -> Self {
			self.attributes = HashMap::from(attributes.map(|t| (t.0.to_string(),t.1.to_string())));
			self
		}
		pub fn contents(mut self, contents: &str) -> Self {
			self.content = Some(contents.to_string());
			self
		}
		pub fn elements<const N: usize>(mut self, elements: [Self; N]) -> Self {
			self.elements = elements.to_vec();
			self
		}
	}
	impl XMLItem {
		pub fn unwrap(self) -> String {
			if let XMLItem::Content(c) = self {c} else {panic!("XMLItem not of content type")}
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
	use super::parser::XMLTree;
	use super::parser::XMLElement;

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
	fn tree_builder(){
		type Elm = XMLElement;
		let tree = XMLTree::try_from("
			<items>
				<food price=\"1\" name=\"apple\"/>
				<drink price=\"2\" name=\"coffee\"/>
			</items>
		".to_string()).unwrap();
		assert_eq!(tree,
//---------------------------------------------------------
XMLTree { elements: vec![
	Elm::builder("items").elements([
		Elm::builder("food").attributes([("price","1"),("name","apple")]),
		Elm::builder("drink").attributes([("price","2"),("name","coffee")]),
	]),
	
]}
//---------------------------------------------------------
		);
	}
}
