use exprparse::Expression;
use std::error::Error;
fn main() -> Result<(),Box<dyn Error>>{
	let args: Vec<_> = std::env::args().collect();
	if args.len() < 2 {
		return Err("Please specify an equation".into())
	}
	println!("{}",Expression::new(&args[1])?.evaluate()?);
	Ok(())
}
