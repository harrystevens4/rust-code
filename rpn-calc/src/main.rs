mod reverse_polish_notation;
use reverse_polish_notation as rpn;
use std::io;
use std::env;

fn main() {
    //====== handle arguments ======
    //skip argv[0] and collect
    let args = env::args().skip(1).collect::<Vec<String>>();
    //map to &str and match
    match args.iter().map(|s| s.as_ref()).collect::<Vec<_>>()[..] {
        ["help"] | ["--help"] => return help(),
        [expression] => eval_expression(expression),
        [] => return eval_from_stdin(),
        _ => return help(),
    };
}

fn help() {
    println!("Usage: rpn-calc <expression>");
    println!("Expression grammar:");
    println!("   - terms seperated by spaces");
    println!("      - \"5 6 +\"");
}

fn eval_from_stdin() {
    println!("Reading from stdin");
    let stateful_evaluator = rpn::StatefulEvaluator::new();
    for line in io::stdin().lines() {
        //unwrap the line
        let line = match line {
            Err(e) => return eprintln!("Error reading stdin: {e}"),
            Ok(line) => line,
        };
        //attempt to parse it
        let expression = match rpn::Expression::parse(line) {
            Ok(expression) => expression,
            Err(e) => {
                eprintln!("=e=> Error parsing expression: {e}");
                continue;
            }
        };
        //attempt to evaluate it
        match expression.evaluate() {
            Ok(result) => println!("===> {result}"),
            Err(e) => {
                eprintln!("=e=> Error evaluating expression: {e}");
                continue;
            }
        };
    }
}

fn eval_expression(expression: &str) {
    //====== parse expresison ======
    let expression = match rpn::Expression::parse(expression) {
        Ok(expression) => expression,
        Err(e) => return eprintln!("Error parsing expression: {e}"),
    };
    //====== evaluate expression ======
    match expression.evaluate() {
        Ok(result) => println!("{result}"),
        Err(e) => eprintln!("Error evaluating expression: {e}"),
    }
}
