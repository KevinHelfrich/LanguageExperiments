use std::env;
use std::io::Read;
use std::fs;

#[derive(PartialEq, Copy, Clone, Debug)]
enum OpCode {
    Left(usize),
    Right(usize),
    Increment(u64),
    Decrement(u64),
    Output(usize),
    Input(usize),
    LoopStart(usize),
    LoopEnd(usize)
}

fn lex(program: String) -> Vec<OpCode> {
    let mut tokens = Vec::new();

    for code in program.chars() {
        let op = match code {
            '<' => Some(OpCode::Left(1)),
            '>' => Some(OpCode::Right(1)),
            '+' => Some(OpCode::Increment(1)),
            '-' => Some(OpCode::Decrement(1)),
            '.' => Some(OpCode::Output(1)),
            ',' => Some(OpCode::Input(1)),
            '[' => Some(OpCode::LoopStart(1)),
            ']' => Some(OpCode::LoopEnd(1)),
            '\n' => None,
            x => { println!("Unexpected token {:?}", x); None }
        };

        match op {
            Some(op) => tokens.push(op),
            None => ()
        }
    }

    tokens
}

fn optimize(program: Vec<OpCode>) -> Vec<OpCode> {
    let mut optimized = Vec::new();
    let mut ip: usize = 1;
    let mut current_op: OpCode = program[0];

    while ip < program.len() {
        match &program[ip] {
            OpCode::Left(x) => {
                match current_op {
                    OpCode::Left(y) => current_op = OpCode::Left(x+y),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::Left(*x);
                    }
                };
            },
            OpCode::Right(x) => {
                match current_op {
                    OpCode::Right(y) => current_op = OpCode::Right(x+y),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::Right(*x);
                    }
                };
            },
            OpCode::Increment(x) => {
                match current_op {
                    OpCode::Increment(y) => current_op = OpCode::Increment(x+y),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::Increment(*x);
                    }
                };
            },
            OpCode::Decrement(x) => {
                match current_op {
                    OpCode::Decrement(y) => current_op = OpCode::Decrement(x+y),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::Decrement(*x);
                    }
                };
            },
            OpCode::Output(x) => {
                match current_op {
                    OpCode::Output(y) => current_op = OpCode::Output(x+y),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::Output(*x);
                    }
                };
            },
            OpCode::Input(x) => {
                match current_op {
                    OpCode::Input(y) => current_op = OpCode::Input(x+y),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::Input(*x);
                    }
                };
            },
            OpCode::LoopStart(_) => {
                match current_op {
                    OpCode::LoopStart(_) => optimized.push(OpCode::LoopStart(0)),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::LoopStart(0);
                    }
                };
            },
            OpCode::LoopEnd(_) => {
                match current_op {
                    OpCode::LoopEnd(_) => optimized.push(OpCode::LoopEnd(0)),
                    _ => {
                        optimized.push(current_op);
                        current_op = OpCode::LoopEnd(0);
                    }
                };
            },
        };
        ip += 1;
    }
    optimized.push(current_op);

    optimized
}

fn optimize_loops(program: Vec<OpCode>) -> Vec<OpCode> {
    let mut optimized = Vec::new();
    let mut ip: usize = 0;

    while ip < program.len() {
        match &program[ip] {
            OpCode::LoopStart(_) => {
                let mut loop_level = 0;
                let mut curr_ip = ip;
                let mut keep_looping = true;
                while keep_looping {
                    curr_ip += 1;
                    match program[curr_ip] {
                        OpCode::LoopEnd(_) => {
                            keep_looping = !(loop_level == 0);
                            loop_level -= 1;
                        },
                        OpCode::LoopStart(_) => loop_level += 1,
                        _ => ()
                    };
                }
                optimized.push(OpCode::LoopStart(curr_ip));
            },
            OpCode::LoopEnd(_) => {
                let mut loop_level = 0;
                let mut curr_ip = ip;
                let mut keep_looping = true;
                while keep_looping {
                    curr_ip -= 1;
                    match program[curr_ip] {
                        OpCode::LoopStart(_) => {
                            keep_looping = !(loop_level == 0);
                            loop_level -= 1;
                        },
                        OpCode::LoopEnd(_) => loop_level += 1,
                        _ => ()
                    };
                }
                optimized.push(OpCode::LoopEnd(curr_ip));
            },
            x => optimized.push(*x),
        }
        ip += 1;
    }
    optimized
}

fn run(program: Vec<OpCode>) {
    let size: usize = 1024*1024*512;
    let mut tape: Vec<u64> = vec![0; size];
    let mut pointer: usize = size/2;
    let mut ip: usize = 0;
    while ip < program.len() {
        let code = &program[ip];
        match code {
            OpCode::Left(x) => pointer -= x,
            OpCode::Right(x) => pointer += x,
            OpCode::Increment(x) => tape[pointer] += x,
            OpCode::Decrement(x) => tape[pointer] -= x,
            OpCode::Output(x) => for _ in 0..*x { print!("{}", (tape[pointer] as u8) as char) },
            OpCode::Input(x) => {
                for _ in 0..*x {
                    let mut input: [u8; 1] = [0; 1];
                    std::io::stdin().read_exact(&mut input).expect("Failed to read input");
                    tape[pointer] = input[0] as u64;
                }
            },
            OpCode::LoopStart(x) => {
                if tape[pointer] == 0  {
                    ip = *x;
                }
            },
            OpCode::LoopEnd(x) => {
                if tape[pointer] > 0  {
                    ip = *x;
                }
            },
        };
        ip += 1;
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

    let optimized = optimize(lexed);

    let super_optimized = optimize_loops(optimized);

    run(super_optimized);
}
