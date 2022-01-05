#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use clap::Parser as CmdParser;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest::prec_climber::Operator;
use pest::prec_climber::PrecClimber;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::mem::size_of;

#[derive(CmdParser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    file: String,

    #[clap(short, long)]
    debug: bool,
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IdentParser;

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

type Register = u8;
type LiteralInteger = i16;

#[derive(Copy, Clone, Debug)]
enum VariableLocation {
    Register(Register),
}

#[derive(Copy, Clone, Debug)]
enum Opcode {
    Add {
        dest: Register,
        in1: Register,
        in2: Register
    },
    Subtract {
        dest: Register,
        in1: Register,
        in2: Register
    },
    Multiply {
        dest: Register,
        in1: Register,
        in2: Register
    },
    Divide {
        dest: Register,
        in1: Register,
        in2: Register
    },
    Power {
        dest: Register,
        in1: Register,
        in2: Register
    },
    LoadLiteral {
        dest: Register,
        value: LiteralInteger
    },
    CopyRegister {
        dest: Register,
        copy_from: Register
    },
    SysCall {
        args: Register,
        function_id: LiteralInteger
    },
    ComparisonEqual {
        dest: Register,
        in1: Register,
        in2: Register
    },
    ComparisonNotEqual {
        dest: Register,
        in1: Register,
        in2: Register
    },
    ComparisonGreaterThan {
        dest: Register,
        in1: Register,
        in2: Register
    },
    ComparisonGreaterThanEqual {
        dest: Register,
        in1: Register,
        in2: Register
    },
    ComparisonLessThan {
        dest: Register,
        in1: Register,
        in2: Register
    },
    ComparisonLessThanEqual {
        dest: Register,
        in1: Register,
        in2: Register
    },
    JumpIfFalseRelative {
        test: Register,
        offset: LiteralInteger
    },
    JumpIfTrueRelative {
        test: Register,
        offset: LiteralInteger
    },
    CopyConstant {
        dest: Register,
        const_lookup: Register
    },
    ArrayAssignment {
        array_reg: Register,
        index: Register,
        value: Register
    },
    ArrayGet {
        array_reg: Register,
        index: Register,
        dest: Register
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
enum Data {
    Number(f64),
    Character(char),
    Boolean(bool),
    Array(RefCell<Vec<Data>>),
}

#[derive(Clone, Debug)]
struct VMState {
    constants: Vec<Data>,
    registers: [Data; 256]
}

fn build_state( compile_data: &mut CompileData) -> VMState {
    const INIT: Data = Data::Number(0.0);
    VMState {
        constants: compile_data.constant_table.clone(),
        registers: [INIT; 256]
    }
}

#[derive(Debug)]
struct CompileData<'a> {
    variable_lookup: HashMap<String, VariableLocation>,
    first_unused_register: Register,
    syscall_name_lookup: &'a mut HashMap<String, LiteralInteger>,
    constant_table: Vec<Data>,
}

fn build_compile_data(syscall_name_lookup: &mut HashMap<String, LiteralInteger>) -> CompileData {
    CompileData {
        variable_lookup: HashMap::new(),
        first_unused_register: 0,
        syscall_name_lookup: syscall_name_lookup,
        constant_table: Vec::new(),
    }
}

fn register_syscalls() -> (HashMap<String, LiteralInteger>, HashMap<LiteralInteger, Box<dyn Fn(Register, &mut VMState)>>) {
    let mut name_lookup = HashMap::new();
    let mut func_defs: HashMap<LiteralInteger, Box<dyn Fn(Register, &mut VMState)>> = HashMap::new();
    let mut current_func_id: LiteralInteger = 0;

    name_lookup.insert("Print".to_string(), current_func_id);
    func_defs.insert(current_func_id, Box::new(|reg: Register, state: &mut VMState| {
        print!("{}", build_print_string(&state.registers[reg as usize]));
    }));
    current_func_id += 1;

    name_lookup.insert("Println".to_string(), current_func_id);
    func_defs.insert(current_func_id, Box::new(|reg: Register, state: &mut VMState| {
        println!("{}", build_print_string(&state.registers[reg as usize]));
    }));
    current_func_id += 1;

    name_lookup.insert("Len".to_string(), current_func_id);
    func_defs.insert(current_func_id, Box::new(|reg: Register, state: &mut VMState| {
        state.registers[reg as usize] = match &state.registers[(reg + 1) as usize] {
            Data::Array(x) => Data::Number(x.borrow().len() as f64),
            _ => panic!("Only array types have a length")
        };
    }));

    (name_lookup, func_defs)
}

fn build_print_string(data: &Data) -> String {
    match data {
        Data::Boolean(x) => format!("{}", x),
        Data::Character(x) => format!("{}", x),
        Data::Number(x) => format!("{}", x),
        Data::Array(x) => {
            let mut s = "".to_string();
            let v = x.borrow();
            for d in v.iter() {
                s += &build_print_string(d);
            }
            s
        },
    }
}

fn execute_program(program : Vec<Opcode>, compile_data: &mut CompileData, syscall_lookups: HashMap<LiteralInteger, Box<dyn Fn(Register, &mut VMState)>>, print_debug: bool) -> VMState {
    let mut state = build_state(compile_data);
    let mut ip: i32 = 0;
    if print_debug {
        println!("Executing...");
    }
    while ip < program.len().try_into().unwrap() {
        if print_debug {
            println!("{}  {:?}", ip, program[ip as usize]);
        }
        match program[ip as usize] {
            Opcode::Add { dest, in1, in2 } => state.registers[dest as usize] = match &state.registers[in1 as usize] {
                Data::Number(x) => match state.registers[in2 as usize] {
                    Data::Number(y) => Data::Number(x + y),
                    _ => panic!("Not a number type as second argument to addition")
                },
                Data::Array(x) => match &state.registers[in2 as usize] {
                        Data::Array(y) => { let mut z = x.borrow().clone(); z.append(&mut y.borrow().clone()); Data::Array(RefCell::new(z)) },
                        y => { let mut z = x.borrow().clone(); z.push(y.clone()); Data::Array(RefCell::new(z)) }
                },
                _ => panic!("Not a number type as first argument to addition")
            },
            Opcode::Subtract { dest, in1, in2 } => state.registers[dest as usize] = match state.registers[in1 as usize] {
                Data::Number(x) => match state.registers[in2 as usize] {
                    Data::Number(y) => Data::Number(x - y),
                    _ => panic!("Not a number type as second argument to subtraction")
                },
                _ => panic!("Not a number type as first argument to subtraction")
            },
            Opcode::Multiply { dest, in1, in2 } => state.registers[dest as usize] = match state.registers[in1 as usize] {
                Data::Number(x) => match state.registers[in2 as usize] {
                    Data::Number(y) => Data::Number(x * y),
                    _ => panic!("Not a number type as second argument to multiplication")
                },
                _ => panic!("Not a number type as first argument to multiplication")
            },
            Opcode::Divide { dest, in1, in2 } => state.registers[dest as usize] = match state.registers[in1 as usize] {
                Data::Number(x) => match state.registers[in2 as usize] {
                    Data::Number(y) => Data::Number(x / y),
                    _ => panic!("Not a number type as second argument to division")
                },
                _ => panic!("Not a number type as first argument to division")
            },
            Opcode::Power { dest, in1, in2 } => state.registers[dest as usize] = match state.registers[in1 as usize] {
                Data::Number(x) => match state.registers[in2 as usize] {
                    Data::Number(y) => Data::Number(f64::powf(x,y)),
                    _ => panic!("Not a number type as second argument to power/exponentiation")
                },
                _ => panic!("Not a number type as first argument to power/exponentiation")
            },
            Opcode::LoadLiteral { dest, value } => state.registers[dest as usize] = Data::Number(value as f64),
            Opcode::CopyRegister { dest, copy_from } => state.registers[dest as usize] = state.registers[copy_from as usize].clone(),
            Opcode::SysCall { args, function_id } => match syscall_lookups.get(&function_id) {
                Some(x) => x(args, &mut state),
                None => panic!("Unrecognized syscall")
            },
            Opcode::ComparisonEqual { dest, in1, in2 } => state.registers[dest as usize] =
                if state.registers[in1 as usize] == state.registers[in2 as usize] { Data::Boolean(true) } else { Data::Boolean(false) },
            Opcode::ComparisonNotEqual { dest, in1, in2 } => state.registers[dest as usize] =
                if state.registers[in1 as usize] != state.registers[in2 as usize] { Data::Boolean(true) } else { Data::Boolean(false) },
            Opcode::ComparisonGreaterThan { dest, in1, in2 } => state.registers[dest as usize] =
                if state.registers[in1 as usize] > state.registers[in2 as usize] { Data::Boolean(true) } else { Data::Boolean(false) },
            Opcode::ComparisonGreaterThanEqual { dest, in1, in2 } => state.registers[dest as usize] =
                if state.registers[in1 as usize] >= state.registers[in2 as usize] { Data::Boolean(true) } else { Data::Boolean(false) },
            Opcode::ComparisonLessThan { dest, in1, in2 } => state.registers[dest as usize] =
                if state.registers[in1 as usize] < state.registers[in2 as usize] { Data::Boolean(true) } else { Data::Boolean(false) },
            Opcode::ComparisonLessThanEqual { dest, in1, in2 } => state.registers[dest as usize] =
                if state.registers[in1 as usize] <= state.registers[in2 as usize] { Data::Boolean(true) } else { Data::Boolean(false) },
            Opcode::JumpIfFalseRelative { test, offset } => match state.registers[test as usize] {
                Data::Boolean(x) => if !x { ip += offset as i32 },
                _ => panic!("non boolean value in JumpIfFalseRelative instruction")
            },
            Opcode::JumpIfTrueRelative { test, offset } => match state.registers[test as usize] {
                Data::Boolean(x) => if x { ip += offset as i32 },
                _ => panic!("non boolean value in JumpIfTrueRelative instruction")
            },
            Opcode::CopyConstant { dest, const_lookup } => state.registers[dest as usize] = match state.constants.get( match state.registers[const_lookup as usize] {
                Data::Number(x) => x as usize,
                _ => panic!("Can only use a number as a lookup for constants!")
            }) {
                Some(x) => x.clone(),
                None => panic!("No constant found")
            },
            Opcode::ArrayAssignment { array_reg, value, index } => match &state.registers[array_reg as usize] {
                Data::Array(x) => {
                    let mut v = x.borrow_mut();
                    v[match state.registers[index as usize] {
                        Data::Number(y) => y as usize,
                        _ => panic!("Must use a number as an index to an array")
                    }] = state.registers[value as usize].clone();
                },
                _ => panic!("Cannot push to a non array")
            },
            Opcode::ArrayGet { index, array_reg, dest } => {
                state.registers[dest as usize] = match &state.registers[array_reg as usize] {
                    Data::Array(x) => {
                        let v = x.borrow();
                         v[ match state.registers[index as usize] {
                            Data::Number(y) => y as usize,
                            _ => panic!("need a number for indexing")
                        }].clone()
                    },
                    _ => panic!("Cannot get data from non array type")
                };
            }
        }
        ip += 1;
    }
    state
}

fn compile_value(value : Pair<Rule>, result_reg: Register, compile_data: &mut CompileData) -> Vec<Opcode> {
    let target = value.into_inner().next().unwrap_or_else(|| panic!("Failed to unwrap val type"));
    let mut opcodes = Vec::new();
    match target.as_rule() {
        Rule::number => {
            opcodes.push(Opcode::LoadLiteral{ dest: result_reg, value: target.as_str().parse::<i16>().unwrap()});
        },
        Rule::ident => {
            let lookup = match compile_data.variable_lookup.get(&target.as_str().to_string()) {
                Some(x) => x,
                None => {
                    panic!("Unknown variable {:?}", target.as_str());
                }
            };
            match lookup {
                VariableLocation::Register(x) => {
                    opcodes.push(Opcode::CopyRegister{ dest: result_reg, copy_from: *x});
                }
            }

        },
        Rule::string => {
            let text = target.into_inner().next().unwrap_or_else(|| panic!("Failed to get string inner")).as_str();
            let raw_data = text.chars().collect::<Vec<_>>();
            let mut data = Vec::new();
            for ch in raw_data {
                data.push(Data::Character(ch));
            }
            compile_data.constant_table.push(Data::Array(RefCell::new(data)));
            opcodes.push(Opcode::LoadLiteral{ dest: result_reg, value: (compile_data.constant_table.len() -1) as i16 });
            opcodes.push(Opcode::CopyConstant{ dest: result_reg, const_lookup: result_reg });
        },
        Rule::arrayElement => {
            let mut arr_inner = target.into_inner();
            let array_name = arr_inner.next().unwrap_or_else(|| panic!("Can't get array name"));
            let array_index = arr_inner.next().unwrap_or_else(|| panic!("Can't get array index"));
            opcodes.push(Opcode::LoadLiteral{ dest: result_reg + 1, value: array_index.as_str().parse::<i16>().unwrap()});
            match compile_data.variable_lookup.get(&array_name.as_str().to_string()) {
                Some(x) => match x {
                    VariableLocation::Register(x) => {
                        opcodes.push(Opcode::ArrayGet{ index: result_reg + 1, array_reg: *x, dest: result_reg });
                    },
                },
                None => panic!("Unknown array")
            };
        },
        _ => unreachable!()
    }
    opcodes
}

fn compile_expression(expression: Pair<Rule>, result_reg: Register, compile_data: &mut CompileData) -> Vec<Opcode> {
    let result = PREC_CLIMBER.climb(
        expression.into_inner(),
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::value => {
                compile_value(pair, result_reg, compile_data)
            },
            Rule::expression => compile_expression(pair, result_reg + 1, compile_data),
            _ => unreachable!(),
        },
        |mut lhs: Vec<Opcode>, op: Pair<Rule>,mut rhs: Vec<Opcode>| match op.as_rule() {
            Rule::add      => {
                let mut opcodes = Vec::new();
                opcodes.append(&mut lhs);
                opcodes.append(&mut rhs);
                opcodes.push(Opcode::Add{ dest: 0, in1: 0, in2: 0 });
                opcodes
            },
            Rule::subtract => {
                let mut opcodes = Vec::new();
                opcodes.append(&mut lhs);
                opcodes.append(&mut rhs);
                opcodes.push(Opcode::Subtract{ dest: 0, in1: 0, in2: 0 });
                opcodes
            },
            Rule::multiply => {
                let mut opcodes = Vec::new();
                opcodes.append(&mut lhs);
                opcodes.append(&mut rhs);
                opcodes.push(Opcode::Multiply{ dest: 0, in1: 0, in2: 0 });
                opcodes
            },
            Rule::divide   => {
                let mut opcodes = Vec::new();
                opcodes.append(&mut lhs);
                opcodes.append(&mut rhs);
                opcodes.push(Opcode::Divide{ dest: 0, in1: 0, in2: 0 });
                opcodes
            },
            Rule::power    => {
                let mut opcodes = Vec::new();
                opcodes.append(&mut lhs);
                opcodes.append(&mut rhs);
                opcodes.push(Opcode::Power{ dest: 0, in1: 0, in2: 0 });
                opcodes
            },
            _ => unreachable!(),
        },
    );
    //The prescedence climbing code doesn't handle register allocation
    //This section runs through the generated code and properly
    //assigns registers to each operation
    let mut reg = result_reg;
    let mut register_result = Vec::new();
    for i in 0..result.len() {
        let instruction = result.get(i);
        match instruction.unwrap() {
             Opcode::LoadLiteral { value, .. } => {
                 register_result.push(Opcode::LoadLiteral { dest: reg, value: *value });
                 reg += 1;
             },
             Opcode::CopyRegister { copy_from, .. } => {
                 register_result.push(Opcode::CopyRegister { dest: reg, copy_from: *copy_from });
                 reg += 1;
             },
             Opcode::CopyConstant { const_lookup, .. } => {
                 reg -= 1; // At the moment CopyReg is allways preceded by LoadLiteral so we need to decrease the reg target
                           //for thinggs to remain correct
                 register_result.push(Opcode::CopyConstant { dest: reg, const_lookup: *const_lookup });
                 reg += 1;
             },
             Opcode::ArrayGet { array_reg, index, ..} => {
                 reg -= 1; // At the moment ArrayGet is allways preceded by LoadLiteral so we need to decrease the reg target
                           //for thinggs to remain correct
                 register_result.push(Opcode::ArrayGet { dest: reg, index: *index, array_reg: *array_reg });
                 reg += 1;
             },
             Opcode::Add {..} => {
                 reg -= 2;
                 register_result.push(Opcode::Add { dest: reg, in1: reg, in2: reg + 1});
                 reg += 1;
             },
             Opcode::Subtract {..} => {
                 reg -= 2;
                 register_result.push(Opcode::Subtract { dest: reg, in1: reg, in2: reg + 1});
                 reg += 1;
             },
             Opcode::Multiply {..} => {
                 reg -= 2;
                 register_result.push(Opcode::Multiply { dest: reg, in1: reg, in2: reg + 1});
                 reg += 1;
             },
             Opcode::Divide {..} => {
                 reg -= 2;
                 register_result.push(Opcode::Divide { dest: reg, in1: reg, in2: reg + 1});
                 reg += 1;
             },
             Opcode::Power {..} => {
                 reg -= 2;
                 register_result.push(Opcode::Power { dest: reg, in1: reg, in2: reg + 1});
                 reg += 1;
             },
             x => panic!("Unexpected OpCode in an expression {:?}", x)
        }
    }
    register_result
}

fn compile_comparison(mut comparison: Pairs<Rule>, compile_data: &mut CompileData) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let lhs = comparison.next().unwrap_or_else(|| panic!("Failed to unwrap expression"));
    let operator = comparison.next().unwrap_or_else(|| panic!("Failed to unwrap operator"));
    let rhs = comparison.next().unwrap_or_else(|| panic!("Failed to unwrap expression"));

    let mut lhs_val = compile_expression(lhs, compile_data.first_unused_register, compile_data);
    let mut rhs_val = compile_expression(rhs, compile_data.first_unused_register + 1, compile_data);

    opcodes.append(&mut lhs_val);
    opcodes.append(&mut rhs_val);

    match operator.as_rule() {
        Rule::boolEqual => opcodes.push(Opcode::ComparisonEqual{ dest: compile_data.first_unused_register,
            in1: compile_data.first_unused_register,
            in2: compile_data.first_unused_register + 1 }),
        Rule::notEqual => opcodes.push(Opcode::ComparisonNotEqual{ dest: compile_data.first_unused_register,
            in1: compile_data.first_unused_register,
            in2: compile_data.first_unused_register + 1 }),
        Rule::greaterThan => opcodes.push(Opcode::ComparisonGreaterThan{ dest: compile_data.first_unused_register,
            in1: compile_data.first_unused_register,
            in2: compile_data.first_unused_register + 1 }),
        Rule::greaterThanEqual => opcodes.push(Opcode::ComparisonGreaterThanEqual{ dest: compile_data.first_unused_register,
            in1: compile_data.first_unused_register,
            in2: compile_data.first_unused_register + 1 }),
        Rule::lessThan => opcodes.push(Opcode::ComparisonLessThan{ dest: compile_data.first_unused_register,
            in1: compile_data.first_unused_register,
            in2: compile_data.first_unused_register + 1 }),
        Rule::lessThanEqual => opcodes.push(Opcode::ComparisonLessThanEqual{ dest: compile_data.first_unused_register,
            in1: compile_data.first_unused_register,
            in2: compile_data.first_unused_register + 1 }),
        _ => panic!("Unrecognised comparison operator: {:?}", operator.as_str())
    };
    opcodes
}

fn compile_while_loop(mut while_loop: Pairs<Rule>, compile_data: &mut CompileData) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let condition = while_loop.next().unwrap_or_else(|| panic!("Failed to unwrap condition"));
    let condition_inner = condition.into_inner();
    let mut res = compile_comparison(condition_inner.clone(), compile_data);
    let block = while_loop.next().unwrap_or_else(|| panic!("Failed to unwrap code block"));
    let mut inner = compile_program(block.into_inner(), compile_data);
    let offset = (inner.len() + res.len()) as i16;

    opcodes.append(&mut res.clone());
    opcodes.push(Opcode::JumpIfFalseRelative{ test: compile_data.first_unused_register, offset: offset + 2 });
    opcodes.append(&mut inner);
    opcodes.append(&mut res);
    let backward = Opcode::JumpIfTrueRelative{ test: compile_data.first_unused_register, offset: -(offset + 1 )};
    opcodes.push(backward);
    opcodes
}

fn compile_if(mut if_statement: Pairs<Rule>, compile_data: &mut CompileData) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let condition = if_statement.next().unwrap_or_else(|| panic!("Failed to unwrap condition"));
    let condition_inner = condition.into_inner();
    let mut res = compile_comparison(condition_inner.clone(), compile_data);
    let block = if_statement.next().unwrap_or_else(|| panic!("Failed to unwrap code block"));
    let mut inner = compile_program(block.into_inner(), compile_data);

    opcodes.append(&mut res);
    opcodes.push(Opcode::JumpIfFalseRelative{ test: compile_data.first_unused_register, offset: inner.len() as i16});
    opcodes.append(&mut inner);
    opcodes
}

fn compile_function_call(mut func_call: Pairs<Rule>, compile_data: &mut CompileData) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let mut arg_codes = Vec::new();
    let func_name = func_call.next().unwrap_or_else(|| panic!("Failed to unwrap function name"));
    let input_args = func_call.next().unwrap_or_else(|| panic!("Couldn't get input args list")).into_inner();
    let mut reg = compile_data.first_unused_register;
    for arg in input_args {
        let code = &mut compile_value(arg, reg, compile_data);
        arg_codes.append( &mut code.clone() );
        opcodes.append( code );
        reg += 1;
    }
    opcodes.push(Opcode::SysCall { args: compile_data.first_unused_register, function_id: match compile_data.syscall_name_lookup.get(&func_name.as_str().to_string()) {
        Some(x) => *x,
        None => panic!("Unknown function {:?}", func_name.as_str())
    }});
    //copy back any values changed by the function
    for code in arg_codes {
        match code {
            Opcode::CopyRegister { dest, copy_from } => opcodes.push(Opcode::CopyRegister { dest: copy_from, copy_from: dest }),
            _ => ()
        };
    }
    opcodes
}

fn compile_assignment(mut assignment: Pairs<Rule>, compile_data: &mut CompileData) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let assignee = assignment.next().unwrap_or_else(|| panic!("Failed to unwrap assignee")).into_inner().next().unwrap_or_else(|| panic!("Failed to get assignment inner"));
    let expression = assignment.next().unwrap_or_else(|| panic!("Failed to unwrap expression"));
    let mut res = compile_expression(expression, compile_data.first_unused_register, compile_data);
    opcodes.append(&mut res);
    match assignee.as_rule() {
        Rule::ident => {
            match compile_data.variable_lookup.get(&assignee.as_str().to_string()) {
                Some(x) => match x {
                    VariableLocation::Register(x) => {
                        opcodes.push(Opcode::CopyRegister{ dest: *x, copy_from: compile_data.first_unused_register});
                    },
                },
                None => {
                    compile_data.variable_lookup.insert(assignee.as_str().to_string(), VariableLocation::Register(compile_data.first_unused_register));
                    compile_data.first_unused_register += 1;
                }
            };
        },
        Rule::arrayElement => {
            let mut arr_inner = assignee.into_inner();
            let array_name = arr_inner.next().unwrap_or_else(|| panic!("Can't get array name"));
            let array_index = arr_inner.next().unwrap_or_else(|| panic!("Can't get array index"));
            opcodes.push(Opcode::LoadLiteral{ dest: compile_data.first_unused_register + 1, value: array_index.as_str().parse::<i16>().unwrap()});
            match compile_data.variable_lookup.get(&array_name.as_str().to_string()) {
                Some(x) => match x {
                    VariableLocation::Register(x) => {
                        opcodes.push(Opcode::ArrayAssignment{ index: compile_data.first_unused_register + 1, array_reg: *x, value: compile_data.first_unused_register });
                    },
                },
                None => {
                    compile_data.variable_lookup.insert(array_name.as_str().to_string(), VariableLocation::Register(compile_data.first_unused_register));
                    compile_data.first_unused_register += 1;
                }
            };
        },
        _ => unreachable!()
    };
    opcodes
}

fn compile_program(program: Pairs<Rule>, compile_data: &mut CompileData) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    for line in program {
        match line.as_rule() {
            Rule::assignment => {
                opcodes.append(&mut compile_assignment(line.into_inner(), compile_data));
            },
            Rule::functionCall => {
                opcodes.append(&mut compile_function_call(line.into_inner(), compile_data));
            },
            Rule::ifStatement => {
                opcodes.append(&mut compile_if(line.into_inner(), compile_data));
            },
            Rule::whileLoop => {
                opcodes.append(&mut compile_while_loop(line.into_inner(), compile_data));
            },
            Rule::EOI => (),
            _ => println!("Unexpected rule {:?}", line.as_rule())
        }
    }
    opcodes
}

fn main() {
    if cfg!(debug_assertions) {
        assert!(size_of::<Opcode>() == 4);
    }

    let args = Args::parse();

    let program_file = args.file;

    let program = fs::read_to_string(program_file)
        .expect("Something went wrong reading the file");

    let pairs = IdentParser::parse(Rule::program, &program).unwrap_or_else(|e| panic!("{}", e));

    let (mut syscall_name_lookup, syscall_lookup) = register_syscalls();

    let mut compile_data = build_compile_data(&mut syscall_name_lookup);

    let code = compile_program(pairs, &mut compile_data);
    let print_debug = args.debug;

    if print_debug {
        let mut i = 0;
        for c in &code {
            println!("{}  {:?}", i, c);
            i += 1;
        }
    }

    execute_program(code, &mut compile_data, syscall_lookup, print_debug);
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
