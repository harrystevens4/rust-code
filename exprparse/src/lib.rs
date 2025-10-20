use std::iter::Peekable;
use std::error::Error;
use std::fmt::{Formatter,Display};
use std::num::ParseFloatError;

#[derive(PartialEq,Debug,Clone)]
enum Lexeme {
	Operator(String),
	Operand(String),
	OpenBrackets(String),
	CloseBrackets(String),
}
#[derive(PartialEq,Debug,Clone)]
struct ExpressionUnit {
	lvalue: Option<Box<Expression>>,
	rvalue: Option<Box<Expression>>,
	operator: String,
}
#[derive(PartialEq,Debug,Clone)]
enum Expression {
	Value(String),
	Expression(ExpressionUnit),
}
#[derive(PartialEq,Debug)]
enum ParseError {
	ExpectedOperator,
	ExpectedOperand,
	UnknownOperator,
}
#[derive(Debug)]
enum EvalError {
	ParseFloatError(ParseFloatError),
	UnknownOperator(String),
}

impl Display for EvalError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>{
		let result = format!("{self:?}");
		let _ = f.write_str(&result);
		Ok(())
	}
}
impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>{
		let result = format!("{self:?}");
		let _ = f.write_str(&result);
		Ok(())
	}
}

impl Error for EvalError {}
impl Error for ParseError {}

impl Lexeme {
	fn unwrap(self) -> String {
		use Lexeme::*;
		match self {
			Operator(s) => s,
			Operand(s) => s,
			OpenBrackets(s) => s,
			CloseBrackets(s) => s,
		}
	}
}

impl Expression {
	pub fn new(expression: &str) -> Result<Self,ParseError> {
		parser(lexer(expression))
	}
	pub fn evaluate(&self) -> Result<f64,EvalError> {
		evaluator(self.clone())
	}
}

fn op_power(operator: Option<Lexeme>) -> Result<i32,ParseError> {
	match operator {
		None => Ok(i32::MIN),
		Some(Lexeme::Operator(operator)) => {
			const OPERATORS: [&str; 5] = ["**","*","/","-","+"];
			OPERATORS.into_iter().rev().position(|x| x == operator).ok_or(ParseError::UnknownOperator).map(|x| x as i32)
		}
		Some(_) => Err(ParseError::ExpectedOperator)
	}
}

fn parser(lexemes: Vec<Lexeme>) -> Result<Expression,ParseError> {
	//transform into array of Expression
	let mut expressions: Vec<ExpressionUnit> = vec![];
	let mut i = 0;
	while i < lexemes.len() {
		use Lexeme::*;
		match &lexemes[i] {
			Operand(op) => { let expr_unit = {
				let left_operator = if i == 0 {None} else {Some(lexemes[i-1].clone())};
				let right_operator = if i == lexemes.len()-1 {None} else {Some(lexemes[i+1].clone())};
				if op_power(left_operator.clone())? < op_power(right_operator.clone())? {
					//account for 2*2 + 3*3 where the plus is empty
					if (expressions.len() > 0 && expressions[expressions.len()-1].rvalue.is_some()) || (expressions.len() == 0 && left_operator.is_some()) {
						expressions.push(
							ExpressionUnit {
								operator: left_operator.ok_or(ParseError::ExpectedOperator)?.unwrap(),
								lvalue: None,
								rvalue: None,
							}
						);
					}
					//right operator is stronger
					ExpressionUnit {
						operator: right_operator.ok_or(ParseError::ExpectedOperator)?.unwrap(),
						lvalue: Some(Box::new(Expression::Value(op.into()))),
						rvalue: None,
					}
				}else{
					let rvalue = Some(Box::new(Expression::Value(op.into())));
					//left operator same or greater than
					if let Some(mut end) = expressions.pop() {
						if end.rvalue.is_none() {
							end.rvalue = rvalue;
							end
						}
						else {
							expressions.push(end);
							ExpressionUnit {
								operator: left_operator.ok_or(ParseError::ExpectedOperator)?.unwrap(),
								lvalue: None,
								rvalue,
							}
						}
					}else {
						ExpressionUnit {
							operator: left_operator.ok_or(ParseError::ExpectedOperator)?.unwrap(),
							lvalue: None,
							rvalue,
						}
					}
				}
			}; expressions.push(expr_unit) },
			_ => (),
		}
		i+=1;
	}
	//evaluate empty expression to 0
	if expressions.len() == 0 { return Ok(Expression::Value("0".to_string())) }
	dbg!(&expressions);
	//absorb adjacent expressions into None lvalues and rvalues
	dbg!(Ok(Expression::Expression(merge_expressions(&expressions[..]))))
}

fn merge_expressions(expressions: &[ExpressionUnit]) -> ExpressionUnit {
	//TODO: expresion units seem to be being lost??
	if expressions.len() == 1 { return expressions[0].clone() }
	//find an expression to start with
	for i in 0..(expressions.len()) {
		//TODO: needs fixing
		//if it is ["7-","8+8"], it start on "8*8" which is already full causing "7-" to be lost
		if (expressions[i].lvalue.is_none() && expressions[i].rvalue.is_none()) || i == expressions.len()-1 {
			let mut new_expression = expressions[i].clone();
			if expressions[i].lvalue.is_none() && i != 0 {
				new_expression.lvalue = Some(Box::new(
					Expression::Expression(merge_expressions(&expressions[..i]))
				));
			}
			if expressions[i].rvalue.is_none() && i != expressions.len()-1 {
				new_expression.rvalue = Some(Box::new(
					Expression::Expression(merge_expressions(&expressions[(i+1)..]))
				));
			}
			return new_expression;
		}
	}
	panic!("Could not find starting expression");
}

fn read_token<T: Fn(char) -> bool, I: Sized+Iterator<Item = char>>(input: &mut Peekable<I>, predicate: T) -> String {
	let mut token = input.next().unwrap().to_string();
	loop {
		if let Some(next) = input.peek() {
			if predicate(*next) { token.push(input.next().unwrap()) }
			else {break}
		}else {break}
	}
	token
}

fn lexer(input_string: &str) -> Vec<Lexeme>{
	let mut result = vec![];
	let mut input = input_string.chars().peekable();
	loop {
		let next_char = if let Some(n) = input.peek() {n} else {break};
		if next_char.is_ascii_digit() || *next_char == '.' { //dont forget about decimals
			//read an operand
			result.push(
				Lexeme::Operand(read_token(
					&mut input,|x| (x.is_ascii_digit()|| x == '.')
				))
			);
		}else if next_char.is_ascii_alphabetic() {
			//read a variable
			let token = read_token(&mut input,|x| x.is_ascii_alphabetic());
			if token.len() > 1 {
				//multi char variables are operators (e.g. function)
				result.push(Lexeme::Operator(token));
			}else{
				//single char variables are operands
				result.push(Lexeme::Operand(token));
			}
		}else if "([{".contains(*next_char) {
			//read brackets
			result.push(
				Lexeme::OpenBrackets(input
					.next()
					.unwrap()
					.to_string()
				)
			);
		}else if ")]}".contains(*next_char) {
			//read brackets
			result.push(
				Lexeme::CloseBrackets(input
					.next()
					.unwrap()
					.to_string()
				)
			);
		}else if next_char.is_ascii_punctuation() {
			//read an operator
			result.push(
				Lexeme::Operator(read_token(
					&mut input,|x| x.is_ascii_punctuation()
				))
			);
		}
	}
	result
}

fn apply_operator(left: f64, right: f64, op: &str) -> Result<f64,EvalError> {
	Ok(match op {
		"*" => left * right,
		"+" => left + right,
		"/" => left / right,
		"-" => left - right,
		"**" => left.powf(right),
		_ => return Err(EvalError::UnknownOperator(op.to_string())),
	})
}

fn parse_operand(op: &str) -> Result<f64,EvalError> {
	use std::str::FromStr;
	f64::from_str(op).map_err(|e| EvalError::ParseFloatError(e))
}

fn evaluator(expr: Expression) -> Result<f64,EvalError> {
	use Expression as Ex;
	if let Ex::Value(value) = expr { return parse_operand(&value) }
	match expr {
		Ex::Value(v) => parse_operand(&v),
		Ex::Expression(e) => {
			let left = match e.lvalue {
				Some(expr) => evaluator(*expr)?,
				None => 0.0
			};
			let right = match e.rvalue {
				Some(expr) => evaluator(*expr)?,
				None => 0.0
			};
			apply_operator(left,right,e.operator.as_str())
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::Lexeme::*;
	use crate::Error;
	use crate::Expression::*;
	use crate::ExpressionUnit;
	use crate::{lexer,parser};

    #[test]
    fn lexer_test(){
        assert_eq!(lexer("sin(x**3)+y"),vec![
			Operator("sin".into()),
			OpenBrackets("(".into()),
			Operand("x".into()),
			Operator("**".into()),
			Operand("3".into()),
			CloseBrackets(")".into()),
			Operator("+".into()),
			Operand("y".into()),
		]);
		assert_eq!(lexer("1.1"),vec![Operand("1.1".into())]);
		assert_eq!(lexer("-42/2"),vec![
			Operator("-".into()),
			Operand("42".into()),
			Operator("/".into()),
			Operand("2".into()),
		]);
    }
	#[test]
	fn evaluator_test() -> Result<(),Box<dyn Error>>{
		use crate::Expression;
		assert_eq!(Expression::new("1+2+3+4+5")?.evaluate()?,15.0);
		assert_eq!(Expression::new("2*5+3*6")?.evaluate()?,28.0);
		assert_eq!(Expression::new("2*5+6/3")?.evaluate()?,12.0);
		assert_eq!(Expression::new("-42")?.evaluate()?,-42.0);
		assert_eq!(Expression::new("-42/2")?.evaluate()?,-21.0);
		assert_eq!(Expression::new("8*8+7-6*9")?.evaluate()?,17.0);
		Ok(())
	}
	#[test]
	fn parser_test(){
		assert_eq!(parser(lexer("")),Ok(Value("0".into())));
		assert_eq!(parser(lexer("-1+2")),Ok(
			Expression(ExpressionUnit {
				lvalue: Some(Box::new(Expression(ExpressionUnit {
					lvalue: None,
					rvalue: Some(Box::new(Value("1".into()))),
					operator: "-".into(),
				}))),
				rvalue: Some(Box::new(Value("2".into()))),
				operator: "+".into(),
			})
		));
		assert_eq!(parser(lexer("-42/2")),Ok(
			Expression(ExpressionUnit {
				operator: "-".into(),
				lvalue: None,
				rvalue: Some(Box::new(Expression(ExpressionUnit {
					operator: "/".into(),
					lvalue: Some(Box::new(Value("42".into()))),
					rvalue: Some(Box::new(Value("2".into()))),
				}))),
			})
		));
	}
}
