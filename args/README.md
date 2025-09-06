
# What is it

Argument processing library for rust

# Provided functions

`Args::new(args: Vec<String>, format: Vec<(Option<&str>,Option<&str>,bool)>) -> Result<Self,ArgError>`
Call this to build an `Arg` struct. For error handling, refer down to the example usage section. Your format should look something like this:
```
let format = vec![
//       short      long               parameter
	(Some("h"), Some("help"),      false    ),
	(Some("n"), Some("no-colour"), false    ),
];
```
Where parameter is if the argument takes a parameter or not.
Warning: it does not remove `args[0]`, so you have to do that yourself

`pub fn has_short(&self, short: &str) -> bool`
checks if a short argument is present

`pub fn has_long(&self, long: &str) -> bool`
checks if a long argument is present

`pub fn get_arg<'a>(&'a self, short_opt: Option<&str>, long_opt: Option<&str>) -> Option<&'a str>`
gets an argument for a short option or a long option, checking long options first. E.g.
```
if let Some(depth_arg) = args.get_arg(Some("d"),Some("depth")) {config.max_depth = depth_arg.parse().unwrap_or(usize::MAX)}
```
Will only return none if neither the short argument or the long argument were present

# Important structures

```
#[derive(Debug,PartialEq)]
pub struct Args {
        //              arg     parameter
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
```

# How to use

## Example usage

```
let format = vec![
//       short      long               parameter
	(Some("h"), Some("help"),      false    ),
	(Some("n"), Some("no-colour"), false    ),
];
let args = match Args::new(env::args().collect(),format){
	Ok(args) => args,
	Err(e) => match e {
		UnknownArgument(t) => {
			eprintln!("Error: unknown argument {:?}",t);
			return;
		},
		MissingParameter(t) => {
			eprintln!("Error: missing parameter to {:?}",t);
			return;
		}
	}
};
if args.has_short("h") || args.has_long("help") {
	print_help();
	return;
}
```

