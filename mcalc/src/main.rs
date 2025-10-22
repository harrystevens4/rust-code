use exprparse::Expression;
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
	let args: Vec<_> = std::env::args().collect();
	if args.len() < 2 {
		interactive_mode();
	}else {
		println!("{}",Expression::new(&args[1])?.evaluate()?);
	}
	Ok(())
}

fn print_help(){
	println!("Usage: {} [options] [equation]",std::env::args().next().unwrap());
	println!("Providing no equation will start interactive mode");
	println!("Interactive mode allows you to use the variable \"a\" as a substitution of the previous answer");
}

fn interactive_mode(){
	let mut prev_answer = 0.0;
	loop {
		let line = match gnu_readline(">>>") { Some(l) => l, None => return, };
		match Expression::new(&line).map(|expr| expr.evaluate()) {
			Ok(Ok(result)) => {
				println!("{result}");
				prev_answer = result;
			},
			Ok(Err(e)) => println!("Error: {e:?}"),
			Err(e) => println!("Error: {e:?}"),
		}
	}
}
