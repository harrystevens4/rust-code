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
    println!("Usage: rpn-calc [expression]");
    println!("Interactive mode:");
    println!("  - when 0 arguments are passed interactive mode is started");
    println!("  - expressions are read from stdin");
    println!("  - after each line the stack is printed");
    println!("    - right most is the top and left most is the bottom");
    println!("  - and any errors are also printed");
    println!("  - `clear` can be used to clear the stack");
    println!("Expression grammar:");
    println!("   - terms seperated by spaces");
    println!("      - \"5 6 +\"");
}

fn eval_from_stdin() {
    println!("Reading from stdin");
    let mut stateful_evaluator = rpn::StatefulEvaluator::new();
    for line in io::stdin().lines() {
        //unwrap the line
        let line = match line.as_ref().map(|s| s.as_ref()) {
            //print any errors
            Err(e) => return eprintln!("Error reading stdin: {e}"),
            //reset the state
            Ok("clear") => {
                stateful_evaluator = rpn::StatefulEvaluator::new();
                println!("===> []");
                continue;
            },
            //continue to processing the expression
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
        match stateful_evaluator.feed(expression) {
            Ok(()) => println!("===> {}", match &stateful_evaluator.stack()[..] {
                [rpn::Term::Value(value)] => format!("{value}"),
                other => format!("{other:?}"),
            }),
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
