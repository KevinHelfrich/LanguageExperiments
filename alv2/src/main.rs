extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use pest::Parser;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::prec_climber::Operator;
use pest::prec_climber::PrecClimber;
use std::env;
use std::fs;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IdentParser;

#[derive(PartialEq, Clone, Debug)]
struct Function {
    args: String,
    function: String
}

fn build_function(args: String, function: String) -> Function {
    Function{
        args: args,
        function: function
    }
}

#[derive(PartialEq, Clone, Debug)]
enum Data {
    Number(f64),
    StringType(String),
    Function(Function),
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left),
            Operator::new(power, Right)
        ])
    };
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: bf <bf program>");
        std::process::exit(1);
    }

    let program_file = args.remove(1);

    let program = fs::read_to_string(program_file)
        .expect("Something went wrong reading the file");

    let pairs = IdentParser::parse(Rule::program, &program).unwrap_or_else(|e| panic!("{}", e));

    let mut scope: HashMap<String, Data> = HashMap::new();
    for pair in pairs {
        evaluate_line(pair, &mut scope);
    }
}

fn evaluate_line<'a>(line: Pair<'a, Rule>, scope: &mut HashMap<String, Data>) {
    match line.as_rule() {
        Rule::assignment => evaluate_assignment(line.into_inner(), scope),
        Rule::functionCall => evaluate_function(line.into_inner(), scope),
        Rule::whileLoop => evaluate_while(line.into_inner(), scope),
        Rule::ifStatement => evaluate_if(line.into_inner(), scope),
        Rule::arrayAssignment => evaluate_array_assignment(line.into_inner(), scope),
        Rule::functionDefiniton => evaluate_function_definition(line.into_inner(), scope),
        Rule::functionDefinitonNoArgs => evaluate_function_definition_no_args(line.into_inner(), scope),
        Rule::load => evaluate_load(line.into_inner(), scope),
        Rule::EOI => (),
        _ => println!("Unexpected rule {:?}", line.as_rule())
    }
}

fn evaluate_function_definition_no_args<'a>(mut func_def: Pairs<'a, Rule>, scope: &mut HashMap<String, Data>) {
    let func_name = func_def.next().unwrap_or_else(|| panic!("Couldn't get function name")).as_str();
    let func_body = func_def.next().unwrap_or_else(|| panic!("Couldn't get function body")).as_str();
    scope.insert(func_name.to_string(), Data::Function(build_function("".to_string(), func_body.to_string())));
}

fn evaluate_function_definition<'a>(mut func_def: Pairs<'a, Rule>, scope: &mut HashMap<String, Data>) {
    let func_name = func_def.next().unwrap_or_else(|| panic!("Couldn't get function name")).as_str();
    let func_args = func_def.next().unwrap_or_else(|| panic!("Couldn't get function args")).as_str();
    let func_body = func_def.next().unwrap_or_else(|| panic!("Couldn't get function body")).as_str();
    scope.insert(func_name.to_string(), Data::Function(build_function(func_args.to_string(), func_body.to_string())));
}

fn evaluate_array_assignment<'a>(mut assignment_statement: Pairs<'a, Rule>, scope: &mut HashMap<String, Data>) {
    let arr_ident = assignment_statement.next().unwrap_or_else(|| panic!("Failed to unwrap condition")).as_str();
    let mut index = 0;
    for p in assignment_statement {
        let lookup = format!("{}_{}", arr_ident, index);
        scope.insert(lookup, Data::Number(p.as_str().parse::<f64>().unwrap()));
        index += 1;
    }

}

fn evaluate_load<'a>(mut load_statement: Pairs<'a, Rule>, scope: &mut HashMap<String, Data>) {
    let file_name = load_statement.next().unwrap_or_else(|| panic!("Couldn't get function name")).as_str();

    let program = fs::read_to_string(format!("{}.al", file_name))
        .expect("Something went wrong reading the file");

    let pairs = IdentParser::parse(Rule::program, &program).unwrap_or_else(|e| panic!("{}", e));

    for pair in pairs {
        evaluate_line(pair, scope);
    }
}

fn evaluate_if<'a>(mut if_statement: Pairs<'a, Rule>, scope: &mut HashMap<String, Data>) {
    let condition = if_statement.next().unwrap_or_else(|| panic!("Failed to unwrap condition"));
    let condition_inner = condition.into_inner();
    let res = evaluate_comparison(condition_inner.clone(), scope);

    if res {
        for p in if_statement {
            evaluate_line(p, scope);
        }
    }
}

fn evaluate_while<'a>(mut while_loop: Pairs<'a, Rule>, scope: &mut HashMap<String, Data>) {
    let condition = while_loop.next().unwrap_or_else(|| panic!("Failed to unwrap condition"));
    let condition_inner = condition.into_inner();
    let mut res = evaluate_comparison(condition_inner.clone(), scope);

    let mut code: Vec<Pair<Rule>> = Vec::new();
    for p in while_loop {
        code.push(p.clone());
    }

    while res {
        for p in 0..code.len() {
            evaluate_line(code[p].clone(), scope);
        }
        res = evaluate_comparison(condition_inner.clone(), scope);
    }
}

fn evaluate_comparison(mut comparison: Pairs<Rule>, scope: &mut HashMap<String, Data>) -> bool {
    let lhs = comparison.next().unwrap_or_else(|| panic!("Failed to unwrap expression"));
    let operator = comparison.next().unwrap_or_else(|| panic!("Failed to unwrap operator"));
    let rhs = comparison.next().unwrap_or_else(|| panic!("Failed to unwrap expression"));

    let lhs_val = match evaluate_expression(lhs, scope) {
        Data::Number(x) => x,
        _ => panic!("Comparison only supports number types"),
    };
    let rhs_val = match evaluate_expression(rhs, scope) {
        Data::Number(x) => x,
        _ => panic!("Comparison only supports number types"),
    };

    match operator.as_rule() {
        Rule::boolEqual => lhs_val == rhs_val,
        Rule::notEqual => lhs_val != rhs_val,
        Rule::greaterThan => lhs_val > rhs_val,
        Rule::greaterThanEqual => lhs_val >= rhs_val,
        Rule::lessThan => lhs_val < rhs_val,
        Rule::lessThanEqual => lhs_val <= rhs_val,
        _ => panic!("Unrecognised comparison operator: {:?}", operator.as_str())
    }
}

fn evaluate_assignment<'a>(mut assignment: Pairs<'a, Rule>, scope: &mut HashMap<String, Data>) {
    let assignee = assignment.next().unwrap_or_else(|| panic!("Failed to unwrap assignee"));
    let expression = assignment.next().unwrap_or_else(|| panic!("Failed to unwrap expression"));
    let res = evaluate_expression(expression, scope);
    match assignee.as_rule() {
        Rule::ident => scope.insert(assignee.as_str().to_string(), res),
        Rule::array => {
            let mut arr_inner = assignee.clone().into_inner();
            let arr_ident = arr_inner.next().unwrap_or_else(|| panic!("Failed to get array name"));
            let arr_value = arr_inner.next().unwrap_or_else(|| panic!("Failed to get array value"));
            let resolved_value = match get_value(arr_value, scope) {
                Data::Number(x) => format!("{}",x),
                Data::StringType(x) => x,
                _ => panic!("Array lookups must be string or number"),
            };
            let lookup = format!("{}_{}", arr_ident.as_str(), resolved_value);
            scope.insert(lookup, res)
        }
        _ => unreachable!(),
    };
}

fn evaluate_expression(expression: Pair<Rule>, scope: &mut HashMap<String, Data>) -> Data {
    PREC_CLIMBER.climb(
        expression.into_inner(),
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::value => get_value(pair, scope),
            Rule::expression => evaluate_expression(pair, scope),
            _ => unreachable!(),
        },
        |lhs: Data, op: Pair<Rule>, rhs: Data| match op.as_rule() {
            Rule::add      => match lhs {
                Data::Number(x) => match rhs {
                    Data::Number(y) => Data::Number(x + y),
                    Data::StringType(y) => Data::StringType(format!("{}{}",x,y)),
                    _ => panic!("Incompaitable types"),
                },
                Data::StringType(x) => match rhs {
                    Data::Number(y) => Data::StringType(format!("{}{}",x,y)),
                    Data::StringType(y) => Data::StringType(format!("{}{}",x,y)),
                    _ => panic!("Incompaitable types"),
                },
                _ => panic!("Inoperable type"),
            },
            Rule::subtract => match lhs {
                Data::Number(x) => match rhs {
                    Data::Number(y) => Data::Number(x - y),
                    _ => panic!("Not a number as second arg to -"),
                }
                _ => panic!("Not a number as first arg to -"),
            },
            Rule::multiply => match lhs {
                Data::Number(x) => match rhs {
                    Data::Number(y) => Data::Number(x * y),
                    _ => panic!("Not a number as second arg to *"),
                }
                _ => panic!("Not a number as first arg to *"),
            },
            Rule::divide   => match lhs {
                Data::Number(x) => match rhs {
                    Data::Number(y) => Data::Number(x / y),
                    _ => panic!("Not a number as second arg to /"),
                }
                _ => panic!("Not a number as first arg to /"),
            },
            Rule::power    => match lhs {
                Data::Number(x) => match rhs {
                    Data::Number(y) => Data::Number(x.powf(y)),
                    _ => panic!("Not a number as second arg to ^"),
                }
                _ => panic!("Not a number as first arg to ^"),
            },
            _ => unreachable!(),
        },
    )
}

fn evaluate_function(mut func_call: Pairs<Rule>, scope: &mut HashMap<String, Data>) {
    let func = func_call.next().unwrap_or_else(|| panic!("Failed to unwrap func type"));
    match func.as_rule() {
        Rule::printLine =>{
            let arg = func_call.next().unwrap_or_else(|| panic!("Failed to unwrap arg type")).into_inner().next().unwrap_or_else(|| panic!("Can't get print argument"));
            let printable = match get_value(arg, scope) {
                Data::Number(x) => format!("{}",x),
                Data::StringType(x) => x,
                _ => panic!("Argument not a printable type"),
            };
            println!("{}", printable);
        },
        Rule::print => {
            let arg = func_call.next().unwrap_or_else(|| panic!("Failed to unwrap arg type")).into_inner().next().unwrap_or_else(|| panic!("Can't get print argument"));
            let printable = match get_value(arg, scope) {
                Data::Number(x) => format!("{}",x),
                Data::StringType(x) => x,
                _ => panic!("Argument not a printable type"),
            };
            print!("{} ", printable);
        },
        Rule::ident => {
            let func = get_function_code(func, scope);
            let mut inner_scope: HashMap<String, Data> = HashMap::new();
            if func.args.chars().count() > 0 {
                let input_args = func_call.next().unwrap_or_else(|| panic!("Couldn't get input args list")).into_inner();
                let args = IdentParser::parse(Rule::arglist, &func.args).unwrap_or_else(|e| panic!("{}", e)).next().unwrap_or_else(|| panic!("Couldn't get function args"));
                let mut a = input_args.clone();
                for p in args.clone().into_inner() {
                    let input_value = a.next().unwrap_or_else(|| panic!("Can't unwrap arg"));
                    let input = get_value(input_value, scope);
                    inner_scope.insert(p.as_str().to_string(), input);
                }
                let pairs = IdentParser::parse(Rule::block, &func.function).unwrap_or_else(|e| panic!("{}", e)).next().unwrap_or_else(|| panic!("Couldn't get function body")).into_inner();
                for pair in pairs {
                    evaluate_line(pair, &mut inner_scope);
                }
                let mut a = input_args.clone();
                for p in args.into_inner() {
                    let input_value = a.next().unwrap_or_else(|| panic!("Can't unwrap arg"));
                    let input = get_value(p, &mut inner_scope);
                    scope.insert(input_value.as_str().to_string(), input);
                }
            } else {
                let pairs = IdentParser::parse(Rule::block, &func.function).unwrap_or_else(|e| panic!("{}", e)).next().unwrap_or_else(|| panic!("Couldn't get function body")).into_inner();
                for pair in pairs {
                    evaluate_line(pair, &mut inner_scope);
                }
            }
        },
        _ => unreachable!(),
    }
}

fn get_function_code<'a>(val: Pair<Rule>, scope: &'a mut HashMap<String, Data>) -> Function {
    match scope.get(val.as_str()) {
        Some(z) => match z {
            Data::Function(x) => x.clone(),
            _ => panic!("{:?} cannot be invoked as a function", val.as_str())
        },
        None => {
            panic!("{:?} is not a known function", val.as_str())
        }
    }
}

fn get_value(val: Pair<Rule>, scope: &mut HashMap<String, Data>) -> Data {
    let mut inn = val.into_inner();
    let target = inn.next().unwrap_or_else(|| panic!("Failed to unwrap val type"));
    match target.as_rule() {
        Rule::ident => {
            return match scope.get(target.as_str()) {
                Some(z) => z.clone(),
                None => {
                    panic!("Unknown variable: {:?}", target.as_str());
                }
            }
        },
        Rule::number => return Data::Number(target.as_str().parse::<f64>().unwrap()),
        Rule::string => {
            let string_inner = target.into_inner().next().unwrap_or_else(|| panic!("can't unwrap string insides"));
            return Data::StringType(string_inner.as_str().to_string())
        },
        Rule::array => {
            let mut arr_inner = target.clone().into_inner();
            let arr_ident = arr_inner.next().unwrap_or_else(|| panic!("Failed to get array name"));
            let arr_value = arr_inner.next().unwrap_or_else(|| panic!("Failed to get array value"));
            let resolved_value = match get_value(arr_value, scope) {
                Data::Number(x) => format!("{}",x),
                Data::StringType(x) => x,
                _ => panic!("Array lookups must be string or number"),
            };
            let lookup = format!("{}_{}", arr_ident.as_str(), resolved_value);
            return match scope.get(lookup.as_str()) {
                Some(z) => z.clone(),
                None => {
                    panic!("Unknown variable: {:?}", target.as_str());
                }
            }
        },
        _ => {
            panic!("Token {} not a valid value type", target.as_str())
        }
    };
}

fn _print_token(token: &Pair<Rule>, depth: i64) {
    match token.as_rule() {
        Rule::digit => return,
        Rule::alpha => return,
        _ => ()
    }
    for _ in 0..depth {
        print!("----");
    }
    println!("Rule:    {:?}", token.as_rule());
    for _ in 0..depth {
        print!("    ");
    }
    println!("Span:    {:?}", token.as_span());
    for _ in 0..depth {
        print!("    ");
    }
    println!("Text:    \"{}\"", token.as_str());
}
