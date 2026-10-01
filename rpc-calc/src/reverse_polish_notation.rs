use std::fmt;
use std::fmt::Formatter;
use std::fmt::Debug;

#[derive(Debug)]
pub enum Term {
	Value(Number),
	Function(Function),
}

#[derive(Debug)]
pub struct Expression {
	terms: Vec<Term>,
}

#[derive(Debug,Clone)]
pub enum ExpressionError {
	ParseError,
	StackEmptyError,
	UnknownFunction(String),
	TooManyValues,
}

type Number = f64;
struct Function {
	function: Box<dyn FnOnce(Number) -> Term>,
}

impl Debug for Function {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error>{
		std::fmt::Display::fmt("<function>",f)
	}
}

impl<T: FnOnce(Number) -> Term + 'static> From<T> for Function {
	fn from(val: T) -> Self {
		Self {
			function: Box::new(val)
		}
	}
}

impl Function {
	//convenience function for fn(Number) -> Number
	pub fn with_1_arg(func: impl FnOnce(Number) -> Number + 'static) -> Self {
		Self {
			function: Box::new(move |term| Term::Value(func(term)))
		}
	}
	//convenience function for fn(Number, Number) -> Number
	pub fn with_2_args(func: impl FnOnce(Number,Number) -> Number + 'static) -> Self {
		let function = move |t1| {
				Term::Function(
					Function::from(move |t2| Term::Value(func(t1,t2)))
				)
		};
		Self {
			function: Box::new(function),
		}
	}
	pub fn apply(self, val: Number) -> Term {
		(self.function)(val)
	}
}

impl Expression {
	pub fn parse(string: impl AsRef<str>) -> Result<Self,ExpressionError> {
		let terms = string
			.as_ref()
			//split by space
			.split(" ")
			.inspect(|term| println!("{term}"))
			//categorise terms as number or function
			.map(|term|{
				//try to parse as float
				if let Ok(number) = term.parse::<Number>() {
					return Ok(Term::Value(number))
				}
				//if not attempt to match with builtin function
				match term {
					"abs" => Ok(Term::Function(
						Function::with_1_arg(|term| term.abs())
					)),
					"-" => Ok(Term::Function(
						Function::with_2_args(|term_1,term_2| term_2-term_1)
					)),
					"+" => Ok(Term::Function(
						Function::with_2_args(|term_1,term_2| term_1+term_2)
					)),
					_ => Err(ExpressionError::UnknownFunction(term.into()))
				}
			})
			.collect::<Result<Vec<_>,ExpressionError>>()?;
		Ok(Self {
			terms,
		})
	}
	fn apply_functions(terms: Vec<Term>) -> Result<Number,ExpressionError> {
		let mut stack: Vec<Term> = vec![];
		match terms[..] {
			//if only one value is left that is the result
			[Term::Value(last_item)] => return Ok(last_item),
			//if only one function is left that is an error
			[Term::Function(_)] => return Err(ExpressionError::StackEmptyError),
			//otherwise continue
			_ => (),
		}
		//if no functions were applied we have an error
		let mut function_applied = false;
		for term in terms {
			dbg!{&term};
			match term {
				Term::Value(val) => stack.push(Term::Value(val)),
				Term::Function(func) => {
					let Some(Term::Value(top)) = stack.pop()
					else {Err(ExpressionError::StackEmptyError)?};
					stack.push(func.apply(top));
					function_applied = true;
				}
			}
		}
		if !function_applied {
			return Err(ExpressionError::TooManyValues)
		}
		Self::apply_functions(stack)
	}
	pub fn evaluate(self) -> Result<Number,ExpressionError> {
		Self::apply_functions(self.terms)
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_9_2() {
		use Term::*;
		let expression = Expression::parse("9 2 +")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,11.0)
    }
    #[test]
    fn sub_9_2() {
		use Term::*;
		let expression = Expression::parse("9 2 -")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,7.0)
    }
    #[test]
    fn sub_2_9() {
		use Term::*;
		let expression = Expression::parse("2 9 -")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,-7.0)
    }
    #[test]
    fn sub_3_4_add_5() {
		use Term::*;
		let expression = Expression::parse("3 4 - 5 +")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,4.0)
    }
    #[test]
    fn abs_minus_15() {
		use Term::*;
		let expression = Expression::parse("-15 abs")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,15.0)
    }
}
