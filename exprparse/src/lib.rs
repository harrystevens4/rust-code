use std::iter::Peekable;

#[derive(PartialEq,Debug)]
enum Lexeme {
	Operator(String),
	Operand(String),
	OpenBrackets(String),
	CloseBrackets(String),
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
		if next_char.is_ascii_digit() {
			//read an operand
			result.push(
				Lexeme::Operand(read_token(
					&mut input,|x| x.is_ascii_digit()
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


#[cfg(test)]
mod tests {
    use super::*;
	use Lexeme::*;

    #[test]
    fn lexer_test() {
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
    }
}
