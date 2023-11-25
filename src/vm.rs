use std::collections::HashMap;
use std::sync::Arc;
use crate::chunk::{OpCode};
use crate::compiler::Compiler;
use crate::value::{FunctionType, Value};
use parking_lot::RwLock;
use crate::scanner::Scanner;
use crate::value;

pub const DEBUG_PRINT_CODE: bool = true;
pub const DEBUG_TRACE_EXECUTION: bool = false;

pub const FRAMES_MAX: usize = 64;
pub const STACK_MAX: usize = FRAMES_MAX * 256;


#[derive(PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    globals: HashMap<String, Value>,
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct CallFrame {
    function: Value,
    ip: usize,
    slots: Vec<Value>,
}

pub fn clock_native(_: Vec<Value>) -> Value {
    Value::Float(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64())
}

pub fn sqrt_native(args: Vec<Value>) -> Value {
    match args[0] {
        Value::Float(f) => Value::Float(f.sqrt()),
        Value::Int(i) => Value::Float((i as f64).sqrt()),
        _ => Value::RunTimeError("Sqrt argument must be a number".to_string()),
    }
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            globals: HashMap::new(),
            frames: Vec::with_capacity(FRAMES_MAX),
            stack: Vec::with_capacity(STACK_MAX),
        };

        vm.define_native("clock".to_string(), Box::new(clock_native), 0);
        vm.define_native("sqrt".to_string(), Box::new(sqrt_native), 1);

        vm
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        self.reset_stack();

        let scanner = Arc::new(RwLock::new(Scanner::new(source)));
        let mut compiler = Compiler::new(FunctionType::Script, scanner);

        let function = compiler.compile();

        let res = match function {
            Some(function) => {
                self.stack.push(Value::Function(function.clone()));
                self.frames.push(CallFrame {
                    function: Value::Function(function.clone()),
                    ip: 0,
                    slots: Vec::with_capacity(function.read().arity),
                });
                InterpretResult::Ok
            }
            None => InterpretResult::CompileError,
        };

        if res == InterpretResult::Ok {
            self.run()
        } else {
            res
        }
    }

    fn binary_op(&mut self, op: OpCode) {
        let b = self.pop().unwrap();
        let a = self.pop().unwrap();

        match (op, a, b) {
            (OpCode::Add, Value::Float(a), Value::Float(b)) => self.push(Value::Float(a + b)),
            (OpCode::Subtract, Value::Float(a), Value::Float(b)) => self.push(Value::Float(a - b)),
            (OpCode::Multiply, Value::Float(a), Value::Float(b)) => self.push(Value::Float(a * b)),
            (OpCode::Divide, Value::Float(a), Value::Float(b)) => self.push(Value::Float(a / b)),
            (OpCode::Greater, Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a > b)),
            (OpCode::Less, Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a < b)),
            (OpCode::Add, Value::Int(a), Value::Float(b)) => self.push(Value::Float(a as f64 + b)),
            (OpCode::Subtract, Value::Int(a), Value::Float(b)) => self.push(Value::Float(a as f64 - b)),
            (OpCode::Multiply, Value::Int(a), Value::Float(b)) => self.push(Value::Float(a as f64 * b)),
            (OpCode::Divide, Value::Int(a), Value::Float(b)) => self.push(Value::Float(a as f64 / b)),
            (OpCode::Greater, Value::Int(a), Value::Float(b)) => self.push(Value::Bool(a as f64 > b)),
            (OpCode::Less, Value::Int(a), Value::Float(b)) => self.push(Value::Bool((a as f64) < b)),
            (OpCode::Add, Value::Float(a), Value::Int(b)) => self.push(Value::Float(a + b as f64)),
            (OpCode::Subtract, Value::Float(a), Value::Int(b)) => self.push(Value::Float(a - b as f64)),
            (OpCode::Multiply, Value::Float(a), Value::Int(b)) => self.push(Value::Float(a * b as f64)),
            (OpCode::Divide, Value::Float(a), Value::Int(b)) => self.push(Value::Float(a / b as f64)),
            (OpCode::Greater, Value::Float(a), Value::Int(b)) => self.push(Value::Bool(a > b as f64)),
            (OpCode::Less, Value::Float(a), Value::Int(b)) => self.push(Value::Bool(a < b as f64)),
            (OpCode::Add, Value::Int(a), Value::Int(b)) => self.push(Value::Int(a + b)),
            (OpCode::Subtract, Value::Int(a), Value::Int(b)) => self.push(Value::Int(a - b)),
            (OpCode::Multiply, Value::Int(a), Value::Int(b)) => self.push(Value::Int(a * b)),
            (OpCode::Divide, Value::Int(a), Value::Int(b)) => self.push(Value::Int(a / b)),
            (OpCode::Greater, Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a > b)),
            (OpCode::Less, Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a < b)),

            (OpCode::Equal, a, b) => self.push(Value::Bool(a == b)),
            (OpCode::Add, Value::String(a), Value::String(b)) => {
                let s = a + &b;
                self.push(Value::String(s));
            }

            (o, a, b) => {
                self.runtime_error(format!("Operands must be two numbers or two strings. Got {:?} {:?} {:?}", o, a, b).as_str());
            }
        }
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction: OpCode = OpCode::from(self.read_byte());

            if DEBUG_TRACE_EXECUTION {
                let frame = self.frames.last().unwrap();
                print!("          ");
                for slot in &frame.slots {
                    print!("[ {} ]", slot);
                }
                println!();

                let function = frame.function.clone();
                match function {
                    Value::Function(ref function) => {
                        let function = function.read();
                        let chunk = function.chunk.read();
                        chunk.disassemble(function.name.as_str());
                    }
                    _ => panic!("Expected function"),
                }
            }

            match instruction {
                OpCode::Return => {
                    let result = self.pop();

                    match result {
                        Some(result) => {
                            let frame = self.frames.pop().unwrap();
                            if self.frames.len() == 0 {
                                self.stack.pop();
                                return InterpretResult::Ok;
                            }

                            let parent_frame = self.frames.last_mut().unwrap();
                            parent_frame.slots.truncate(parent_frame.slots.len() - frame.slots.len());
                            self.push(result);
                        }
                        None => {
                            self.runtime_error("Stack underflow");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::Negate => {
                    let value = self.pop().unwrap();
                    match value {
                        Value::Int(value) => self.push(Value::Int(-value)),
                        Value::Float(value) => self.push(Value::Float(-value)),
                        _ => {
                            self.runtime_error("Operand must be a number");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Equal => self.binary_op(OpCode::Equal),
                OpCode::Greater => self.binary_op(OpCode::Greater),
                OpCode::Less => self.binary_op(OpCode::Less),
                OpCode::Add => self.binary_op(OpCode::Add),
                OpCode::Subtract => self.binary_op(OpCode::Subtract),
                OpCode::Multiply => self.binary_op(OpCode::Multiply),
                OpCode::Divide => self.binary_op(OpCode::Divide),
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::Not => {
                    let value = self.pop().unwrap();
                    self.push(Value::Bool(value.is_falsely()));
                }
                OpCode::Print => {
                    println!("{}", self.pop().unwrap());
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::DefineGlobal => {
                    let constant = self.read_constant();
                    let name = constant.to_string();
                    let value = self.pop().unwrap();
                    self.globals.insert(name, value);
                }
                OpCode::GetGlobal => {
                    let constant = self.read_constant();
                    let name = constant.to_string();
                    let value = self.globals.get(&name);

                    match value {
                        Some(value) => self.push(value.clone()),
                        None => {
                            self.runtime_error(format!("Undefined variable '{}'", name).as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::SetGlobal => {
                    let constant = self.read_constant();
                    let name = constant.to_string();
                    if self.globals.contains_key(&name) {
                        let value = self.pop().unwrap();
                        self.globals.insert(name, value);
                    } else {
                        self.runtime_error(format!("Undefined variable '{}'", name).as_str());
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte();
                    let value = self.frames.last().unwrap().slots[slot as usize].clone();
                    self.push(value);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte();
                    let value = self.peek(0).unwrap().clone();
                    self.frames.last_mut().unwrap().slots[slot as usize] = value;
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short();
                    if self.peek(0).unwrap().is_falsely() {
                        self.frames.last_mut().unwrap().ip += offset as usize;
                    }
                }
                OpCode::JumpIfTrue => {
                    let offset = self.read_short();
                    if !self.peek(0).unwrap().is_falsely() {
                        self.frames.last_mut().unwrap().ip += offset as usize;
                    }
                }
                OpCode::Jump => {
                    let offset = self.read_short();
                    self.frames.last_mut().unwrap().ip += offset as usize;
                }
                OpCode::Loop => {
                    let offset = self.read_short();
                    self.frames.last_mut().unwrap().ip -= offset as usize;
                }
                OpCode::Duplicate => {
                    if let Some(value) = self.peek(0) {
                        self.push(value.clone());
                    } else {
                        self.runtime_error("Stack underflow");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Call => {
                    let arg_count = self.read_byte();
                    if !self.call_value(self.peek(arg_count as usize).unwrap().clone(), arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
            }
        }
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> bool {
        match callee {
            Value::Function(function) => {
                self.call(function, arg_count);
                true
            }
            Value::NativeFunction(function) => {
                let function = function.read();
                if arg_count != function.arity as u8 {
                    self.runtime_error(format!("Expected {} arguments but got {}", function.arity, arg_count).as_str());
                    return false;
                }

                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.pop().unwrap());
                }

                args.reverse();

                let result = (function.function)(args);

                match result {
                    Value::RunTimeError(s) => {
                        self.runtime_error(s.as_str());
                        false
                    }
                    _ => {
                        self.push(result);
                        true
                    }
                }
            }
            _ => {
                self.runtime_error("Can only call functions and classes");
                false
            }
        }
    }

    fn call(&mut self, function: Arc<RwLock<value::Function>>, arg_count: u8) {
        if arg_count != function.read().arity as u8 {
            self.runtime_error(format!("Expected {} arguments but got {}", function.read().arity, arg_count).as_str());
            return;
        }

        let frame = self.frames.last_mut().unwrap();
        let mut slots = frame.slots.split_off(frame.slots.len() - arg_count as usize);

        self.frames.push(CallFrame {
            function: Value::Function(function.clone()),
            ip: 0,
            slots,
        });
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);

        for frame in self.frames.iter().rev() {
            match frame.function {
                Value::Function(ref function) => {
                    let function = function.read();
                    let chunk = function.chunk.read();
                    let line = chunk.lines[frame.ip - 1];
                    eprintln!("[line {}] in {}", line, function.name);
                }
                _ => panic!("Expected function"),
            }
        }

        let mut frame = self.frames.last_mut().unwrap();

        self.stack.truncate(frame.slots.len());

        if self.frames.len() == 1 {
            let frame = self.frames.last_mut().unwrap();
            match frame.function {
                Value::Function(ref function) => {
                    frame.ip = function.read().chunk.read().code.len() - 1;
                }
                _ => panic!("Expected function"),
            }
        } else {
            self.frames.pop();
            frame = self.frames.last_mut().unwrap();
            frame.ip += 1;
        }
    }

    fn define_native(&mut self, name: String, function: Box<fn(Vec<Value>) -> Value>, arity: usize) {
        self.stack.push(Value::String(name.clone()));
        let native_function = Arc::new(RwLock::new(value::NativeFunction::new(name.clone(), arity, function)));
        self.stack.push(Value::NativeFunction(native_function.clone()));
        self.globals.insert(name.clone(), Value::NativeFunction(native_function));
        self.stack.pop();
        self.stack.pop();
    }

    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut();
        match frame {
            Some(frame) => {
                match frame.function {
                    Value::Function(ref function) => {
                        let function = function.read();
                        let byte = function.chunk.read().code[frame.ip];
                        frame.ip += 1;
                        byte
                    }
                    _ => panic!("Expected function"),
                }
            }
            None => panic!("Expected frame"),
        }
    }

    #[inline(always)]
    fn read_constant(&mut self) -> Value {
        let frame = self.frames.last_mut().unwrap();
        match frame.function {
            Value::Function(ref function) => {
                let function = function.read();
                let constant = function.chunk.read().code[frame.ip];
                frame.ip += 1;
                let chunk = function.chunk.read();
                chunk.constants[constant as usize].clone()
            }
            _ => panic!("Expected function"),
        }
    }

    #[inline(always)]
    fn read_short(&mut self) -> u16 {
        let frame = self.frames.last_mut().unwrap();
        match frame.function {
            Value::Function(ref function) => {
                let function = function.read();
                let byte1 = function.chunk.read().code[frame.ip].clone();
                let byte2 = function.chunk.read().code[frame.ip + 1].clone();
                frame.ip += 2;
                (byte1 as u16) << 8 | (byte2 as u16)
            }
            _ => panic!("Expected function"),
        }
    }

    #[inline(always)]
    fn push(&mut self, value: Value) {
        self.frames.last_mut().unwrap().slots.push(value);
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<Value> {
        self.frames.last_mut().unwrap().slots.pop()
    }

    #[inline(always)]
    fn peek(&self, distance: usize) -> Option<&Value> {
        let len = self.frames.last().unwrap().slots.len();
        self.frames.last().unwrap().slots.get(len - 1 - distance)
    }
}
