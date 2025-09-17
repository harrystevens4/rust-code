//======================= library =======================
#[derive(Debug,PartialEq)]
pub struct Args {
	//          arg  parameter
	pub short: Vec<(String,Option<String>)>,
	pub long: Vec<(String,Option<String>)>,
	pub other: Vec<String>,
}
#[derive(Debug,Clone,PartialEq)]
pub enum ArgError {
	UnknownArgument(ArgType),
	MissingParameter(ArgType),
}
#[derive(Debug,Clone,PartialEq)]
pub enum ArgType {
	Other(String),
	Short(String),
	Long(String),
}
impl Args {
	//format is passed a tuple like this:
	//       short         long              parameter
	// vec![(Some("h"),    Some("help"),     false    ),
	//      (None,         Some("width"),    true     )]
	//passing a parameter looks like this:
	// `--width 20` or `-w 20`
	pub fn new(args: Vec<String>, format: Vec<(Option<&str>,Option<&str>,bool)>) -> Result<Self,ArgError> {
		let arg_has_parameter = move |arg: &ArgType|{
			for arg_info in &format{
				match arg {
					ArgType::Short(ref arg) => {
						if arg_info.0 == Some(&arg) {return Some(arg_info.2)}
					},
					ArgType::Long(ref arg) => {
						if arg_info.1 == Some(&arg) {return Some(arg_info.2)}
					},
					ArgType::Other(_) => {
						return Some(false) //other argumens never have a parameter
					},
				}
			}
			None
		};
		let confirm_is_arg = |arg: Option<String>,param|{
			if arg.is_none() {Err(ArgError::MissingParameter(param))}
			else {Ok(arg)}
		};
		//====== initialise ======
		let mut args_struct = Args {short: vec![], long: vec![], other: vec![]};
		let mut arg_iter = args.into_iter();
		loop {
			//====== next arg ======
			let arg = match arg_iter.next(){
				Some(a) => a,
				None => break,
			};
			//====== anything past "--" is treated as an other arg ======
			if arg == "--"{
				args_struct.other.extend(&mut arg_iter);
				break;
			};
			//====== classify the argument type ======
			match Self::classify(arg){
				//====== short ======
				ArgType::Short(arg) => {
					//split up all the args
					for ch in arg.chars(){
						if arg_has_parameter(&ArgType::Short(ch.into())).ok_or(ArgError::UnknownArgument(ArgType::Short(ch.into())))?{
							args_struct.short.push(
								(ch.to_string(),confirm_is_arg(arg_iter.next(),ArgType::Short(arg.clone()))?)
							);
						}else{
							args_struct.short.push( (ch.to_string(),None) );
						}
					}
					()
				},
				//====== long ======
				ArgType::Long(arg) => {
					if arg_has_parameter(&ArgType::Long(arg.clone()))
						.ok_or(ArgError::UnknownArgument(ArgType::Long(arg.clone())))?{
						args_struct.long.push(
							(arg.to_string(),confirm_is_arg(arg_iter.next(),ArgType::Long(arg.clone()))?)
						);
					}else{
						args_struct.long.push( (arg,None) )
					}
				},
				//====== other ======
				ArgType::Other(arg) => {
					args_struct.other.push(arg.clone());
				},
			};
		};
		Ok(args_struct)
	}
	//returns relevant argtype with leading '-'s stripped off
	pub fn classify(arg: String) -> ArgType {
		// '-' counts as an other
		// "--" is dealt with in new
		// "-asdf" and "-b" are shorts
		// "--hello" is a long
		// "900" is an other
		if arg.len() == 0 || arg.len() == 1 {ArgType::Other(arg)}
		else if arg[..2] == *"--" {ArgType::Long(arg[2..].into())}
		else if arg[..1] == *"-" {ArgType::Short(arg[1..].into())}
		else {ArgType::Other(arg)}
	}
	pub fn has_long(&self, long: &str) -> bool {
		for arg in &self.long {
			if arg.0 == long {return true}
		}
		false
	}
	pub fn has_short(&self, short: &str) -> bool {
		for arg in &self.short {
			if arg.0 == short {return true}
		}
		false
	}
	pub fn has(&self, short: &str, long: &str) -> bool{
		return self.has_long(long) || self.has_short(short)
	}
	pub fn get_arg<'a>(&'a self, short_opt: Option<&str>, long_opt: Option<&str>) -> Option<&'a str>{
		if let Some(long) = long_opt {
			for arg in &self.long {
				if arg.0 == long {return arg.1.as_deref()}
			}
		}
		if let Some(short) = short_opt {
			for arg in &self.short {
				if arg.0 == short {return arg.1.as_deref()}
			}
		}
		None
	}
}
//======================= tests =======================
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn example_values_test(){
		let format =  vec![
			(Some("p"),None,true),
			(Some("h"),Some("help"),false),
			(Some("a"),None,true),
			(Some("b"),None,true),
			(Some("c"),None,true),
		];
		let sample_args = vec![
			"--help","localhost", "8765", "-pabc", "999", "q", "w", "e"
		].into_iter().map(|x| x.to_string()).collect();
		let args = Args::new(sample_args,format).unwrap();
		assert_eq!(args.other,vec!["localhost","8765"]);
		assert_eq!(args.short,vec![
			("p",Some("999")),
			("a",Some("q")),
			("b",Some("w")),
			("c",Some("e")),
		].into_iter().map(|x| (x.0.to_string(),x.1.map(|s| s.to_string())) ).collect::<Vec<(String,Option<String>)>>());
		assert_eq!(args.long,vec![(String::from("help"),None)]);
	}
	#[test]
	fn bad_argument_test(){
		use crate::ArgError::UnknownArgument;
		use crate::ArgType::Long;
		let format =  vec![
			(Some("p"),None,true),
			(Some("h"),Some("help"),false),
		];
		let sample_args = vec![
			"--help","localhost", "8765", "-p", "999", "what", "--thing"
		].into_iter().map(|x| x.to_string()).collect();
		let args = Args::new(sample_args,format);
		assert_eq!(args,Err(UnknownArgument(Long("thing".to_string()))));
		/*assert_eq!(args.other,vec!["localhost","8765","what"]);
		assert_eq!(args.short,vec![(String::from("p"),Some(String::from("999")))]);
		assert_eq!(args.long,vec![(String::from("help"),None)]);*/
	}
	#[test]
	fn missing_long_parameter_test(){
		use crate::ArgError::MissingParameter;
		use crate::ArgType::Long;
		let format =  vec![
			(Some("p"),None,true),
			(Some("h"),Some("help"),false),
			(None,Some("opt"),true)
		];
		let sample_args = vec![
			"--help","localhost", "8765", "-p", "999", "what", "--opt"
		].into_iter().map(|x| x.to_string()).collect();
		let args = Args::new(sample_args,format);
		assert_eq!(args,Err(MissingParameter(Long("opt".to_string()))));
	}
	fn missing_short_parameter_test(){
		todo!();
	}
}
