use std::str::FromStr;
use parser::XMLParseError;
use std::collections::HashMap;
use composer::*;
//====== structs used in both modules =====
#[derive(Debug,PartialEq,Clone)]
pub struct XMLTree {
	pub version: String,
	pub standalone: bool,
	pub encoding: String,
	pub elements: Vec<XMLElement>,
}
#[derive(Debug,PartialEq,Clone)]
pub struct XMLElement {
	name: String,
	attributes: HashMap<String,String>,
	content: Option<String>,
	elements: Vec<XMLElement>,
}
impl XMLTree {
	pub fn from_elements(elements: Vec<XMLElement>) -> Self {
		XMLTree {
			version: "1.0".to_string(),
			encoding: "UTF-8".to_string(),
			standalone: false,
			elements,
		}
	}
	/*pub fn find_elements<'s>(&'s self) -> &'s XMLElement {

	}*/
	pub fn as_string(&self, add_declaration: bool) -> String {
		let mut tokens = vec![];
		let standalone = {
			if self.standalone {"yes"}
			else {"no"}
		}.to_string();
		if add_declaration {tokens.push(format!("<?xml version=\"{}\" encoding=\"{}\" standalone=\"{}\"?>",self.version,self.encoding,standalone))}
		for element in &self.elements {
			tokens.append(&mut element_to_tokens(element));
		}
		tokens.into_iter().collect()
	}
}
impl FromStr for XMLTree {
	type Err = XMLParseError;
	fn from_str(s: &str) -> Result<Self,Self::Err>{
		return XMLTree::try_from(s.to_string());
	}
}
impl From<XMLTree> for String {
	fn from(tree: XMLTree) -> String {
		tree.as_string(true)
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
//====== parser ======
mod parser {
	use super::*;
	use std::collections::HashMap;
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
	
	//====== structures ======
	//provided as the interface to the user
	#[derive(Debug,PartialEq,Clone)]
	pub enum XMLParseError {
		TagNeverClosed(usize), //position
		EmptyTag(usize),
		MalformedAttriubute(usize),
		MalformedTag(usize),
		NoEndTag(String), //the index into all the tags
		UnexpectedEndTag(String),
		UnexpectedContent(String),
	}
	//used by the lexer
	#[derive(Debug,PartialEq,Clone)]
	pub enum XMLTagType {
		Start,
		End,
		Empty,
		ProcessingInstruction,
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
	fn parse_attributes(attribute_list: Vec<String>, _overall_position: usize) -> Result<HashMap<String,String>,XMLParseError>{
		let mut attributes = HashMap::new();
		let mut current_position = 0;
		for attr in attribute_list
			.into_iter()
			.map(|s| s.trim_matches('/').trim_matches('?').to_string())
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
				'?' => match first_char {
					'?' => XMLTagType::ProcessingInstruction,
					_ => return Err(XMLParseError::MalformedTag(position)),
				},
				_ => match first_char {
					'/' => XMLTagType::End,
					_ => XMLTagType::Start,
				}
			},
			name: items.iter().next().ok_or(XMLParseError::EmptyTag(position))?.trim_matches('/').trim_matches('?').to_string(),
			attributes: parse_attributes(
				items.into_iter().map(|s| s.to_string()).collect::<Vec<String>>()[1..].into(),position
			)?,
		})
	}
	pub fn lexer(data: String) -> Result<Vec<XMLItem>,XMLParseError>{
		//------ utility closure ------
		/*let get_until_matches = |end: &str|{
			let tag = String::new();
			let start_position = position + 1;
			//consume the comment
			let mut buffer = vec![];
			//grab len to start off with
			for _i in 0..end.len() {
				buffer.push(data_iter.next().ok_or(XMLParseError::TagNeverClosed(start_position))?);
				position += 1;
			}
			//keep looking one by one
			loop{
				if buffer.iter().collect::<String>() == end {break}
				buffer.push(data_iter.next().ok_or(XMLParseError::TagNeverClosed(start_position))?);
				tag.push(buffer.remove(0));
				position += 1;
			}
		}*/
		//-----------------------------
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
						Empty | ProcessingInstruction => XMLElement { name: current_token.name.clone(), attributes: current_token.attributes.clone(), content: None, elements: vec![] },
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
			let mut version = "1.0".to_string();
			let mut encoding = "UTF-8".to_string();
			let mut standalone = false;
			let mut elements = build_from_tokens(lexer(data)?)?;
			if elements.len() > 0 && elements[0].name.to_lowercase() == "xml" {
				let declaration = elements.remove(0);
				if let Some(e) = declaration.attributes.get("encoding") {encoding = e.clone()}
				if let Some(v) = declaration.attributes.get("version") {version = v.clone()}
				if let Some(s) = declaration.attributes.get("standalone") {standalone = s == "yes"}
			}
			Ok(XMLTree {
				version,
				encoding,
				standalone,
				elements,
			})
		}
	}
	impl XMLItem {
		pub fn unwrap(self) -> String {
			if let XMLItem::Content(c) = self {c} else {panic!("XMLItem not of content type")}
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
}
mod composer {
	use super::*;
	pub fn hashmap_to_attr_string(map: &HashMap<String,String>) -> String {
		let mut result = String::new();
		for key in map.keys(){
			result.push_str(format!("{}=\"{}\" ",key,map[key]).as_str());
		}
		result
	}
	pub fn element_to_tokens(element: &XMLElement) -> Vec<String>{
		let mut result: Vec<String> = vec![];
		//====== add the start tag ======
		let attributes = hashmap_to_attr_string(&element.attributes);
		let start_tag = format!("<{} {}",element.name,attributes).trim_end().to_string() + ">";
		result.push(start_tag);
		//====== add any content ======
		if let Some(content) = &element.content {
			result.push(content.to_string());
		}
		//====== add any contained elements ======
		for sub_element in &element.elements {
			result.append(&mut element_to_tokens(&sub_element));
		}
		//====== add the end tag ======
		let end_tag = format!("</{}>",&element.name);
		result.push(end_tag);
		result
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
	use super::XMLTree;
	use super::XMLElement;

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
				<drink price=\"99\" name=\"tea\"/>
				<bag>
					<abacus/>
					<aardvark/>
					<ambulance/>
				</bag>
			</items>
		".to_string()).unwrap();
		assert_eq!(tree,
		XMLTree::from_elements(vec![
			Elm::builder("items").elements([
				Elm::builder("food").attributes([("price","1"),("name","apple")]),
				Elm::builder("drink").attributes([("price","2"),("name","coffee")]),
				Elm::builder("drink").attributes([("price","99"),("name","tea")]),
				Elm::builder("bag").elements([
					Elm::builder("abacus"),
					Elm::builder("aardvark"),
					Elm::builder("ambulance"),
				]),
			]),
			
		])
		);
	}
	#[test]
	fn xml_declaration(){
		let tree = "
			<?XmL version=\"9.9\" encoding=\"UTF-8\" random=\"bogus\" standalone=\"yes\"?>
			<apple/>
			<banana/>
		".parse::<XMLTree>().unwrap();
		assert_eq!(tree,
		XMLTree {
			version: "9.9".to_string(),
			encoding: "UTF-8".to_string(),
			standalone: true,
			elements: vec![
				XMLElement::builder("apple"),
				XMLElement::builder("banana"),
			]
		}
		);
	}
	#[test]
	fn tree_as_string(){
		let tree = XMLTree {
			version: "1.0".to_string(),
			encoding: "UTF-8".to_string(),
			standalone: true,
			elements: vec![
				XMLElement::builder("apple"),
				XMLElement::builder("basket").elements([
					XMLElement::builder("pear").attributes([("size","1")]),
					XMLElement::builder("orange").contents("ripe"),
				]),
				XMLElement::builder("banana"),
			]
		};
		assert_eq!(tree.as_string(false),"<apple></apple><basket><pear size=\"1\"></pear><orange>ripe</orange></basket><banana></banana>")
	}
	#[test]
	fn string_from_tree(){
		let tree = XMLTree {
			version: "1.0".to_string(),
			encoding: "UTF-8".to_string(),
			standalone: true,
			elements: vec![
				XMLElement::builder("computer").contents("mine"),
				XMLElement::builder("watches").elements([
					XMLElement::builder("omega"),
					XMLElement::builder("rolex"),
				]),
			]
		};
		assert_eq!(Into::<String>::into(tree),"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><computer>mine</computer><watches><omega></omega><rolex></rolex></watches>")
	}
}
