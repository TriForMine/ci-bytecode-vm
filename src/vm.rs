use crate::chunk::{Chunk, OpCode};
use crate::compiler::Compiler;

const DEBUG_TRACE_EXECUTION: bool = true;

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM<'a> {
    chunk: &'a mut Chunk,
    ip: usize,
    stack: Vec<f64>,
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        VM {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.chunk.clear();
        self.ip = 0;
        self.stack.clear();
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        self.reset();

        let mut compiler = Compiler::new(self.chunk);
        if !compiler.compile(&source) {
            return InterpretResult::CompileError;
        }

        self.run()
    }

    fn binary_op(&mut self, op: OpCode) {
        let b = self.pop().unwrap();
        let a = self.pop().unwrap();
        match op {
            OpCode::Add => self.push(a + b),
            OpCode::Subtract => self.push(a - b),
            OpCode::Multiply => self.push(a * b),
            OpCode::Divide => self.push(a / b),
            _ => panic!("Unknown opcode {}", op),
        }
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction: OpCode = OpCode::from(self.read_byte());

            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {} ]", slot);
                }
                println!();
                self.chunk.disassemble("test chunk");
            }

            match instruction {
                OpCode::Return => {
                    let value = self.pop();
                    println!("{:?}", value);
                    return InterpretResult::Ok;
                },
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                },
                OpCode::Negate => {
                    let value = self.pop().unwrap();
                    self.push(-value);
                },
                OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide => {
                    self.binary_op(instruction);
                },
                _ => {
                    println!("Unknown opcode {}", instruction);
                    return InterpretResult::RuntimeError;
                }
            }
        }
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    #[inline]
    fn read_constant(&mut self) -> f64 {
        let constant = self.read_byte();
        self.chunk.constants[constant as usize]
    }

    #[inline]
    fn push(&mut self, value: f64) {
        self.stack.push(value);
    }

    #[inline]
    fn pop(&mut self) -> Option<f64> {
        self.stack.pop()
    }
}
