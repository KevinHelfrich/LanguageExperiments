use std::env;
use std::fs;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone, Debug)]
enum LexType {
    Name,
    Operator,
    Number
}

#[derive(PartialEq, Clone, Debug)]
enum Lexeme {
    Print,
    JumpEZ,
    Label,
    Variable(String),
    Equals,
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    EndOfLine,
    Number(String),
    OpenSquareBracket,
    CloseSquareBracket,
    ArrInit,
    ArrAssign,
    ArrRead
}

fn get_lexeme(token: &str, parse_type: LexType) -> Lexeme {
    match parse_type {
        LexType::Name => {
            match token {
                "Print" | "print" => return Lexeme::Print,
                "JumpEZ" | "jumpEZ" => return Lexeme::JumpEZ,
                "Label" | "label" => return Lexeme::Label,
                "ArrInit" | "arrInit" => return Lexeme::ArrInit,
                "ArrAssign" | "arrAssign" => return Lexeme::ArrAssign,
                "ArrRead" | "arrRead" => return Lexeme::ArrRead,
                _ => return Lexeme::Variable(token.to_string())
            }
        },
        LexType::Number => {
            return Lexeme::Number(token.to_string());
        },
        LexType::Operator => {
            match token {
                "=" => return Lexeme::Equals,
                "+" => return Lexeme::Plus,
                "-" => return Lexeme::Minus,
                "*" => return Lexeme::Times,
                "/" => return Lexeme::Divide,
                ";" => return Lexeme::EndOfLine,
                "^" => return Lexeme::Power,
                "[" => return Lexeme::OpenSquareBracket,
                "]" => return Lexeme::CloseSquareBracket,
                &_  => {
                    println!("Unexpected nonsense! {:?}", token);
                    return Lexeme::Equals;
                }
            }
        }
    }
}

fn lex(program: String) -> Vec<Lexeme> {
    let mut tokens = Vec::new();
    let mut curr: String = "".to_string();
    let mut curr_type = LexType::Name;

    for code in program.chars() {
        match code {
            'A'..='Z' | 'a' ..= 'z' => {
                match curr_type {
                    LexType::Name => curr.push(code),
                    _ => {
                        if curr.chars().count() > 0 {
                            tokens.push(get_lexeme(curr.as_str(),curr_type)); 
                            curr = "".to_string();
                        }
                        curr.push(code);
                        curr_type = LexType::Name;
                    }
                }
            },
            ' ' | '\n' | '\r' => {
                if curr.chars().count() > 0 {
                    tokens.push(get_lexeme(curr.as_str(),curr_type)); 
                    curr = "".to_string();
                }
            },
            '=' | '+' | '-' | '*' | '/' | ';' | '^' | '[' | ']' => { 
                if curr.chars().count() > 0 {
                    tokens.push(get_lexeme(curr.as_str(),curr_type)); 
                    curr = "".to_string();
                }
                curr.push(code);
                tokens.push(get_lexeme(curr.as_str(),LexType::Operator)); 
                curr = "".to_string();
            },
            '0'..='9' => {
                match curr_type {
                    LexType::Number => curr.push(code),
                    _ => {
                        if curr.chars().count() > 0 {
                            tokens.push(get_lexeme(curr.as_str(),curr_type)); 
                            curr = "".to_string();
                        }
                        curr.push(code);
                        curr_type = LexType::Number;
                    }
                }
            },
            x => println!("Unexpected character {:?}", x)
        };
    }

    tokens
}

fn interpret(lexes: Vec<Lexeme>) {
    let mut scope: HashMap<&str, f64> = HashMap::new();
    let mut labels: HashMap<&str, usize> = HashMap::new();
    let mut ip = 0;

    while ip < lexes.len() {
        let token = &lexes[ip];
        match token {
            Lexeme::Label => {
                let label_name = match &lexes[ip+1] {
                    Lexeme::Variable(x) => x,
                    x => {
                        println!("Not a valid label name: {:?}", x);
                        "none"
                    }
                };
                labels.insert(label_name, ip+3); // (label) (name) (;)
            },
            _ => ()
        }
        ip += 1;
    }

    ip = 0;
    while ip < lexes.len() {
        let lhs = &lexes[ip];
        match lhs {
            Lexeme::Print => {
                let print_var = &lexes[ip+1];
                match print_var {
                    Lexeme::Number(x) => println!("{:?}", x),
                    Lexeme::Variable(x) => {
                        match scope.get(x.as_str()) {
                            Some(y) => println!("{:?}", y),
                            None => println!("Unknown variable: {:?}", x)
                        }
                    },
                    _ => println!("Unprintable : {:?}", print_var)
                }
                ip += 3; // (print) (x) (;)
            },
            Lexeme::JumpEZ => {
                let target = match &lexes[ip+1] {
                    Lexeme::Variable(x) => x,
                    x => {
                        println!("Not a valid label name: {:?}", x);
                        "none"
                    }
                };
                let var = match &lexes[ip+2] {
                    Lexeme::Variable(x) => x,
                    x => {
                        println!("Not a valid label name: {:?}", x);
                        "none"
                    }
                };
                let val: f64 = match scope.get(var) {
                    Some(y) => *y,
                    None => {
                        println!("Unknown variable: {:?}", var);
                        0.0
                    }
                };
                if val == 0.0 {
                    let jump_location: usize = match labels.get(target){
                        Some(y) => *y,
                        None => {
                            println!("Unkown label: {:?}", target);
                            0
                        }
                    };
                    ip = jump_location;
                } else {
                    ip += 4; // (jumpEZ) (Label) (Var) (;)
                }
            },
            Lexeme::Label => {
                //label locations are precalculated and stored in labels dictionary
                ip += 3; // (label) (name) (;)
            },
            Lexeme::Variable(x) => {
                let first = match &lexes[ip+2] {
                    Lexeme::Number(y) => y.parse::<f64>().unwrap(),
                    Lexeme::Variable(y) => {
                        match scope.get(y.as_str()) {
                            Some(z) => *z,
                            None => {
                                println!("Unknown variable: {:?}", y);
                                1.0
                            }
                        }
                    },
                    x => {
                        println!("Unexpected: {:?}", x);
                        0.0
                    }
                };
                let operator = &lexes[ip+3];
                let second = match &lexes[ip+4] {
                    Lexeme::Number(y) => y.parse::<f64>().unwrap(),
                    Lexeme::Variable(y) => {
                        match scope.get(y.as_str()) {
                            Some(z) => *z,
                            None => {
                                println!("Unknown variable: {:?}", y);
                                1.0
                            }
                        }
                    },
                    y => {
                        println!("Unexpected: {:?}", y);
                        0.0
                    }
                };
                let result = match operator {
                    Lexeme::Plus => first + second,
                    Lexeme::Minus => first - second,
                    Lexeme::Times => first * second,
                    Lexeme::Divide => first / second,
                    Lexeme::Power => first.powf(second),
                    y => {
                        println!("Unexpected: {:?}", y);
                        0.0
                    }
                };
                scope.insert(x.as_str(), result);
                ip += 6 // (var) (=) (1) (+) (2) (;)
            },
            x => println!("Unacceptable LHS: {:?}, {:?}", x, ip)
        }
    }
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

    let lexed = lex(program);

    interpret(lexed);
}
