use exprparse::Expression;
use args::Args;
use std::error::Error;
use std::ffi::*;

#[link(name = "readline")]
unsafe extern "C" {
	fn readline(prompt: *const c_char) -> *const c_char;
	fn free(ptr: *const c_void);
	fn add_history(line: *const c_char);
}

fn gnu_readline(prompt: &str) -> Option<String> {
	let raw_prompt = CString::new(prompt).unwrap();
	let line = unsafe { readline(raw_prompt.as_ptr()) };
	if line.is_null() {return None}
	unsafe { add_history(line) };
	let line_owned = unsafe { CStr::from_ptr(line) }
		.to_string_lossy()
		.to_string();
	unsafe { free(line as *const c_void) };
	Some(line_owned)
}

fn main() -> Result<(),Box<dyn Error>>{
	let args = Args::new(std::env::args().collect(),vec![
		(Some("h"), Some("help"), false),
	])?;
	if args.has("h","help") {
		print_help();
		return Ok(());
	}
	if args.other.len() < 2 {
		interactive_mode();
	}else {
		println!("{}",Expression::new(&args.other[0])?.evaluate()?);
	}
	Ok(())
}

fn print_help(){
	println!("Usage: {} [options] [expression]",std::env::args().next().unwrap());
	println!("Providing no equation will start interactive mode");
	println!("Interactive mode allows you to use the variable \"a\" as a substitution of the previous answer\nand entering an empty line causes it to re run the previous expression");
}

fn interactive_mode(){
	use std::collections::HashMap;
	let mut vars = HashMap::from([("a".to_string(),0.0)]);
	let mut prev_expression = String::new();
	loop {
		let mut line = match gnu_readline(">>>") { Some(l) => l, None => return, };
		if line.is_empty() {
			line = prev_expression.clone();
		}else {
			prev_expression = line.clone();
		}
		match Expression::new(&line).map(|expr| expr.evaluate_with_substitution(&vars)) {
			Ok(Ok(result)) => {
				println!("{result}");
				let _ = vars.get_mut("a").map(|a| *a = result);
			},
			Ok(Err(e)) => println!("Error: {e:?}"),
			Err(e) => println!("Error: {e:?}"),
		}
	}
}
