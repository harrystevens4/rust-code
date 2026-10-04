use std::fmt;
use std::fmt::{Formatter,Debug,Display};

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
	StackEmptyError,
	UnknownFunction(String),
	TooManyValues,
	NotEnoughValues,
}

pub struct StatefulEvaluator {
    stack: Vec<Number>,
}

type Number = f64;

//Function is a function that takes in a number and returns either a function or number
//Functions that take more than one argument can be built by manually currying or using
//the helper `with_2_args()` function
pub struct Function {
	function: Box<dyn FnOnce(Number) -> Term>,
}

impl Display for ExpressionError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error>{
        use ExpressionError::*;
        let message = match self {
            UnknownFunction(f) => format!("Unknown function \"{f}\""),
            StackEmptyError => format!("Stack was empty when pop() attempted"),
            TooManyValues => format!("Result of evaluation has more than one value"),
            NotEnoughValues => format!("Function called on empty stack"),
        };
		std::fmt::Display::fmt(&format!("{}",message),f)
	}
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
            //remove empty terms
            .filter_map(|string|
                if string.is_empty() {None}
                else {Some(string)}
            )
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
						Function::with_2_args(|term_1,term_2| term_2+term_1)
					)),
					"*" => Ok(Term::Function(
						Function::with_2_args(|term_1,term_2| term_2*term_1)
					)),
					"/" => Ok(Term::Function(
						Function::with_2_args(|term_1,term_2| term_2/term_1)
					)),
                    "mod" => Ok(Term::Function(
						Function::with_2_args(|term_1,term_2| term_2.rem_euclid(term_1))
                    )),
					_ => Err(ExpressionError::UnknownFunction(term.into()))
				}
			})
			.collect::<Result<Vec<_>,ExpressionError>>()?;
		Ok(Self {
			terms,
		})
	}
    fn terms(self) -> Vec<Term> {
        self.terms
    }
	pub fn evaluate(self) -> Result<Number,ExpressionError> {
        //just a wrapper around the stateful evaluator
        let mut evaluator = StatefulEvaluator::new();
        evaluator.feed(self)?;
        evaluator.result()
    }
}

impl StatefulEvaluator {
    pub fn new() -> Self {
        Self {
            stack: vec![],
        }
    }
    pub fn feed(&mut self, expression: Expression) -> Result<(),ExpressionError> {
		//VecDequeue would probably be more suitable
		let mut reverse_terms: Vec<Term> = expression
            .terms()
            .into_iter()
            .rev()
            .collect();
		//loop through our terms
		loop {
			match reverse_terms.pop() {
				//if we find a value add it to the stack
				Some(Term::Value(val)) => {
					self.stack.push(val);
				}
				//if we find a function, apply it and add the result
				//to the end of reverse_terms
				Some(Term::Function(func)) => {
					let Some(top) = self.stack.pop()
					else {Err(ExpressionError::NotEnoughValues)?};
					reverse_terms.push(func.apply(top));
					//using a function such as `+` (which is curried) will
					//result in it being partially applied, then added back to the terms
					//so that it can be applied again on the next iteration to result in
					//a value that will then be added to the stack in the iteration after
				},
				//nothing left means computation complete
				None => break Ok(())
			}
		}
    }
    pub fn stack<'a>(&'a self) -> &'a Vec<Number> {
        &self.stack
    }
    pub fn result(self) -> Result<Number,ExpressionError> {
		//last item in the stack should be the result
		match self.stack[..] {
			//if only one value is left that is the result
			[last_item] => Ok(last_item),
			//no values means not enough expressions
			[] => Err(ExpressionError::StackEmptyError),
			_ => Err(ExpressionError::TooManyValues),
		}
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
    fn add_3_4_add_5_6_mult() {
		use Term::*;
		let expression = Expression::parse("3 4 + 5 6 + *")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,77.0)
    }
	#[test]
    fn div_10_5() {
		use Term::*;
		let expression = Expression::parse("10 5 /")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,2.0)
    }
    #[test]
    fn complex_1() {
		use Term::*;
		let expression = Expression::parse("2 3 4 + * 10 5 / -")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,12.0)
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
    #[test]
    fn complex_2() {
		use Term::*;
		let expression = Expression::parse("15 7 1 1 + - / 3 * 2 1 1 + + -")
			.unwrap();
		let result = expression.evaluate()
			.unwrap();
		assert_eq!(result,5.0)
    }
}
