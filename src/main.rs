use std::io::Write;
use std::sync::Arc;
use crate::chunk::{Chunk};
use parking_lot::RwLock;

mod chunk;
mod debug;
mod vm;
mod scanner;
mod token_type;
mod compiler;
mod parser_rules;
mod value;

fn repl(vm: &mut vm::VM) {
    loop {
        print!("> ");
        std::io::stdout().flush().expect("Failed to flush stdout");

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("Failed to read line");

        vm.interpret(line);
    }
}

fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).expect("Failed to read file")
}

fn run_file(path: &str, vm: &mut vm::VM) {
    let source = read_file(path);

    let result = vm.interpret(source);

    match result {
        vm::InterpretResult::Ok => std::process::exit(0),
        vm::InterpretResult::CompileError => std::process::exit(65),
        vm::InterpretResult::RuntimeError => std::process::exit(70),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let chunk = Arc::new(RwLock::new(Chunk::new()));
    let mut vm = vm::VM::new(chunk);

    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        run_file(&args[1], &mut vm);
    } else {
        println!("Usage: rlox [path]");
        std::process::exit(64);
    }
}
