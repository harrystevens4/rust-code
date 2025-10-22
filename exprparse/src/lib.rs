#![allow(dead_code)]
use std::iter::Peekable;
use std::error::Error;
use std::fmt::{Formatter,Display};
use std::num::ParseFloatError;

#[derive(PartialEq,Debug,Clone)]
pub enum Lexeme {
	Operator(String),
	Operand(String),
	OpenBrackets(String),
	CloseBrackets(String),
}
#[derive(PartialEq,Debug,Clone)]
pub struct ExpressionUnit {
	lvalue: Option<Box<Expression>>,
	rvalue: Option<Box<Expression>>,
	operator: String,
}
#[derive(PartialEq,Debug,Clone)]
pub enum Expression {
	Value(String),
	Expression(ExpressionUnit),
}
#[derive(PartialEq,Debug)]
pub enum ParseError {
	ExpectedOperator,
	ExpectedOperand,
	UnknownOperator,
	UnexpectedValue,
	UnexpectedParenthesis,
	UnclosedParenthesis,
}
#[derive(Debug)]
pub enum EvalError {
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

fn op_power(expression: Option<ExpressionUnit>) -> Result<i32,ParseError> {
	match expression {
		None => Ok(i32::MIN),
		Some(expr) => {
			const OPERATORS: [&str; 5] = ["**","*","/","-","+"];
			OPERATORS.into_iter().rev().position(|x| x == expr.operator).ok_or(ParseError::UnknownOperator).map(|x| x as i32)
		}
		//Some(_) => Err(ParseError::ExpectedOperator)
	}
}

fn lexemes_to_expressions(lexemes: Vec<Lexeme>) -> Result<Vec<Expression>,ParseError> {
	use Lexeme::*;
	let mut i = 0;
	let mut transformed_lexemes = vec![];
	while i < lexemes.len() {
		match &lexemes[i] {
			OpenBrackets(_) => {
				//collect lexemes between brackets
				let mut sub_expression = vec![];
				//find the matching pair of closing brackets
				i+=1;
				let mut depth = 1;
				loop {
					if i == lexemes.len() {Err(ParseError::UnclosedParenthesis)?}
					match &lexemes[i] {
						OpenBrackets(_) => depth += 1,
						CloseBrackets(_) => depth -= 1,
						_ => sub_expression.push(lexemes[i].clone()),
					}
					if depth == 0 {break} //closing bracket
					i+=1;
				}
				transformed_lexemes.push(parser(sub_expression)?);
			},
			CloseBrackets(_) => Err(ParseError::UnexpectedParenthesis)?,
			Operand(op) => transformed_lexemes.push(Expression::Value(op.into())),
			Operator(op) => transformed_lexemes.push(Expression::Expression(ExpressionUnit {
				operator: op.into(), lvalue: None, rvalue: None
			})),
		}
		i += 1;
	}
	Ok(transformed_lexemes)
}

fn parser(lexemes: Vec<Lexeme>) -> Result<Expression,ParseError> {
	use Lexeme::*;
	//TODO: rewrite this so it turns the lexemes into Expression::Value or Expression::Expression({lvalue: None, rvalue: None, operator})
	//maybe moving the bracket substitution out here and creating a vec of Expression before processing the operand absorbtion
	//====== transform into array of Expression ======
	let mut transformed_lexemes: Vec<Expression> = lexemes_to_expressions(lexemes)?;
	//====== merge adjacent Values into Expressions ======
	let mut expressions: Vec<Expression> = vec![];
	let mut i = 0;
	while i < transformed_lexemes.len() {
		use Expression as Exp;
		//if its not a value or a full ExpressionUnit skip
		match &transformed_lexemes[i] {
			Exp::Expression(expr) => {
				if expr.lvalue.is_none() || expr.rvalue.is_none() {
					i+=1;
					continue
				}
			},
			_ => (),
		}
		//decide which operator to add it to (left or right)
		let expr = {
			let left_operator = 
				if i == 0 {None} 
				else {Some(match transformed_lexemes[i-1].clone(){
						Exp::Expression(expr) => expr,
						Exp::Value(v) => Err(ParseError::UnexpectedValue)?,
				})};
			let right_operator =
				if i == transformed_lexemes.len()-1 {None} 
				else {Some(match transformed_lexemes[i+1].clone(){
						Exp::Expression(expr) => expr,
						Exp::Value(v) => Err(ParseError::UnexpectedValue)?,
				})};
			let operand = transformed_lexemes[i].clone();
			if left_operator.is_none() && right_operator.is_none() {
				operand
			}else if op_power(left_operator.clone())? < op_power(right_operator.clone())? {
				//account for 2*2 + 3*3 where the plus is empty
				let last_expression_rvalue = {
					if let Some(Exp::Expression(expr)) = expressions.iter().last() {
						expr.rvalue.clone()
					}else {None}
				};
				if last_expression_rvalue.is_some() || (expressions.len() == 0 && left_operator.is_some()) {
					expressions.push(Exp::Expression(
						ExpressionUnit {
							operator: left_operator.ok_or(ParseError::ExpectedOperator)?.operator,
							lvalue: None, rvalue: None,
						}
					));
				}
				//right operator is stronger
				Exp::Expression(ExpressionUnit {
					operator: right_operator.ok_or(ParseError::ExpectedOperator)?.operator,
					lvalue: Some(Box::new(operand)),
					rvalue: None,
				})
			}else{
				let rvalue = Some(Box::new(operand));
				//left operator same or greater than
				let last_expression = expressions.iter().last().to_owned();
				if let Some(Exp::Expression(end)) = last_expression {
					if end.rvalue.is_none() {
						let mut new_end = end.clone();
						new_end.rvalue = rvalue;
						let _ = expressions.pop();
						Exp::Expression(new_end)
					}
					else {
						Exp::Expression(ExpressionUnit {
							operator: left_operator.ok_or(ParseError::ExpectedOperator)?.operator,
							lvalue: None, rvalue,
						})
					}
				}else {
					Exp::Expression(ExpressionUnit {
						operator: left_operator.ok_or(ParseError::ExpectedOperator)?.operator,
						lvalue: None, rvalue,
					})
				}
			}
		};
		expressions.push(expr);
		i+=1;
	}
	//evaluate empty expression to 0
	if expressions.len() == 0 { return Ok(Expression::Value("0".to_string())) }
	//dbg!(&expressions);
	//absorb adjacent expressions into None lvalues and rvalues
	//dbg!(&expressions);
	//====== build a tree ======
	Ok(merge_expressions(&expressions[..])?)
}

fn merge_expressions(expressions: &[Expression]) -> Result<Expression,ParseError> {
	//base case
	if expressions.len() == 1 { return Ok(expressions[0].clone()) }
	if let Expression::Expression(expr) = &expressions[0] && expr.rvalue.is_none(){
		//absorb the epxression to the right
		let mut new_expression = expr.clone();
		new_expression.rvalue = Some(Box::new(merge_expressions(&expressions[1..])?));
		Ok(Expression::Expression(new_expression))
	}else if let Expression::Expression(expr) = &expressions[1] && expr.lvalue.is_none(){
		//the expression to the right absorbs us
		let mut new_expressions: Vec<Expression> = expressions[1..]
			.into_iter()
			.map(|x| x.clone())
			.collect();
		match &mut new_expressions[0] {
			Expression::Expression(ex) => ex.lvalue = Some(Box::new(expressions[0].clone())),
			Expression::Value(_) => Err(ParseError::UnexpectedValue)?,
		};
		merge_expressions(&new_expressions[..])
	}else {Err(ParseError::ExpectedOperator)}
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
					&mut input,|x| x.is_ascii_digit() || x == '.'
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
					&mut input,|x| x.is_ascii_punctuation() && !("{[()]}".contains(x))
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
		assert_eq!(lexer("7+(1*2)"),vec![
			Operand("7".into()),
			Operator("+".into()),
			OpenBrackets("(".into()),
			Operand("1".into()),
			Operator("*".into()),
			Operand("2".into()),
			CloseBrackets(")".into()),
		]);
		assert_eq!(lexer("(1*2)"),vec![
			OpenBrackets("(".into()),
			Operand("1".into()),
			Operator("*".into()),
			Operand("2".into()),
			CloseBrackets(")".into()),
		]);
		assert_eq!(lexer("(8)+1"),vec![
			OpenBrackets("(".into()),
			Operand("8".into()),
			CloseBrackets(")".into()),
			Operator("+".into()),
			Operand("1".into()),
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
		assert_eq!(Expression::new("3*(3+5)")?.evaluate()?,24.0);
		assert_eq!(Expression::new("(8)+1")?.evaluate()?,9.0);
		assert_eq!(Expression::new("3*(3+5)+7")?.evaluate()?,31.0);
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
		assert_eq!(parser(lexer("3*(3+5)")),Ok(
			Expression(ExpressionUnit {
				operator: "*".into(),
				lvalue: Some(Box::new(Value("3".into()))),
				rvalue: Some(Box::new(Expression(ExpressionUnit {
					operator: "+".into(),
					lvalue: Some(Box::new(Value("3".into()))),
					rvalue: Some(Box::new(Value("5".into()))),
				}))),
			})
		));
	}
}
