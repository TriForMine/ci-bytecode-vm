use crate::chunk::OpCode;
use crate::compiler::Compiler;
use crate::scanner::Scanner;
use crate::value;
use crate::value::{FunctionType, Value};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;

pub const DEBUG_PRINT_CODE: bool = true;
pub const DEBUG_TRACE_EXECUTION: bool = false;

pub const FRAMES_MAX: usize = 64;
pub const STACK_MAX: usize = 256;

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
    closure: Rc<RwLock<value::Closure>>,
    ip: usize,
    slots: Vec<Value>,
}

pub fn clock_native(_: Vec<Value>) -> Value {
    Value::Float(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
    )
}

pub fn sqrt_native(args: Vec<Value>) -> Value {
    match args[0] {
        Value::Float(f) => Value::Float(f.sqrt()),
        Value::Int(i) => Value::Float((i as f64).sqrt()),
        _ => Value::RunTimeError("Sqrt argument must be a number".to_string()),
    }
}

pub fn input_native(_: Vec<Value>) -> Value {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    Value::String(input.trim().to_string())
}

pub fn throw_native(args: Vec<Value>) -> Value {
    Value::RunTimeError(args[0].to_string())
}

pub fn open_file_native(args: Vec<Value>) -> Value {
    match &args[0] {
        Value::String(s) => match std::fs::File::open(s.clone()) {
            Ok(file) => {
                let mut file = std::io::BufReader::new(file);
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Failed to read file");

                Value::String(contents)
            }
            Err(_) => Value::RunTimeError(format!("Failed to open file '{}'", s)),
        },
        _ => Value::RunTimeError("Expected string".to_string()),
    }
}

pub fn exit_native(args: Vec<Value>) -> Value {
    match args[0] {
        Value::Int(i) => std::process::exit(i as i32),
        _ => Value::RunTimeError("Expected int".to_string()),
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
        vm.define_native("input".to_string(), Box::new(input_native), 0);
        vm.define_native("throw".to_string(), Box::new(throw_native), 1);
        vm.define_native("open".to_string(), Box::new(open_file_native), 1);
        vm.define_native("exit".to_string(), Box::new(exit_native), 1);

        vm
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        self.reset_stack();

        let scanner = Rc::new(RwLock::new(Scanner::new(source)));
        let mut compiler = Compiler::new(FunctionType::Script, scanner);

        let function = compiler.compile();

        let res = match function {
            Some(function) => {
                let closure = Rc::new(RwLock::new(value::Closure::new(function.clone())));

                self.stack.pop();

                self.frames.push(CallFrame {
                    closure: closure.clone(),
                    ip: 0,
                    slots: Vec::with_capacity(STACK_MAX),
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
            (OpCode::Subtract, Value::Int(a), Value::Float(b)) => {
                self.push(Value::Float(a as f64 - b))
            }
            (OpCode::Multiply, Value::Int(a), Value::Float(b)) => {
                self.push(Value::Float(a as f64 * b))
            }
            (OpCode::Divide, Value::Int(a), Value::Float(b)) => {
                self.push(Value::Float(a as f64 / b))
            }
            (OpCode::Greater, Value::Int(a), Value::Float(b)) => {
                self.push(Value::Bool(a as f64 > b))
            }
            (OpCode::Less, Value::Int(a), Value::Float(b)) => {
                self.push(Value::Bool((a as f64) < b))
            }
            (OpCode::Add, Value::Float(a), Value::Int(b)) => self.push(Value::Float(a + b as f64)),
            (OpCode::Subtract, Value::Float(a), Value::Int(b)) => {
                self.push(Value::Float(a - b as f64))
            }
            (OpCode::Multiply, Value::Float(a), Value::Int(b)) => {
                self.push(Value::Float(a * b as f64))
            }
            (OpCode::Divide, Value::Float(a), Value::Int(b)) => {
                self.push(Value::Float(a / b as f64))
            }
            (OpCode::Greater, Value::Float(a), Value::Int(b)) => {
                self.push(Value::Bool(a > b as f64))
            }
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
                self.runtime_error(
                    format!(
                        "Operands must be two numbers or two strings. Got {:?} {:?} {:?}",
                        o, a, b
                    )
                    .as_str(),
                );
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

                frame
                    .closure
                    .read()
                    .function
                    .read()
                    .chunk
                    .read()
                    .disassemble(frame.closure.read().function.clone().read().name.as_str());
            }

            match instruction {
                OpCode::Invoke => {
                    let method = self.read_constant();
                    let arg_count = self.read_byte();
                    if !self.invoke(method, arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Closure => {
                    let constant = self.read_constant();
                    let function = match constant {
                        Value::Function(function) => function,
                        _ => panic!("Expected function"),
                    };
                    let closure = value::Closure::new(function.clone());

                    for _ in 0..function.read().up_value_count {
                        let is_local = self.read_byte() == 1;
                        let index = self.read_byte();
                        if is_local {
                            closure.up_values.write().push(self.capture_up_value(
                                self.frames.last().unwrap().slots[index as usize].clone(),
                            ));
                        } else {
                            closure.up_values.write().push(
                                self.frames.last().unwrap().closure.read().up_values.read()
                                    [index as usize]
                                    .clone(),
                            );
                        }
                    }

                    self.push(Value::Closure(Rc::new(RwLock::new(closure))));
                }
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
                            parent_frame
                                .slots
                                .truncate(parent_frame.slots.len() - frame.slots.len());

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
                OpCode::GetUpvalue => {
                    let slot = self.read_byte();
                    let value = self.frames.last().unwrap().closure.read().up_values.read()
                        [slot as usize]
                        .read()
                        .location
                        .clone();
                    self.push(value);
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte();
                    let value = self.peek(0).unwrap().clone();
                    self.frames
                        .last_mut()
                        .unwrap()
                        .closure
                        .read()
                        .up_values
                        .read()[slot as usize]
                        .write()
                        .location = value;
                }
                OpCode::CloseUpvalue => {
                    self.close_up_values();
                    self.pop();
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
                OpCode::Class => {
                    let name = self.read_constant();
                    self.push(Value::Class(Rc::new(RwLock::new(value::Class::new(
                        name.to_string(),
                    )))));
                }
                OpCode::GetProperty => {
                    let name = self.read_constant();
                    let value = self.pop().unwrap();
                    match value {
                        Value::Instance(ref instance) => {
                            if let Some(value) =
                                instance.read().fields.read().get(&name.to_string())
                            {
                                self.push(value.clone());
                            } else if !self.bind_method(Rc::new(RwLock::new(value.clone())), name) {
                                return InterpretResult::RuntimeError;
                            }
                        }
                        _ => {
                            self.runtime_error("Only instances have properties");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::SetProperty => {
                    let name = self.read_constant();
                    let instance = self.peek(1).unwrap().clone();
                    match instance {
                        Value::Instance(instance) => {
                            let value = self.peek(0).unwrap().clone();
                            instance
                                .write()
                                .fields
                                .write()
                                .insert(name.to_string(), value);
                        }
                        _ => {
                            self.runtime_error("Only instances have fields");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Method => {
                    let name = self.read_constant();
                    self.define_method(name);
                }
            }
        }
    }

    fn invoke(&mut self, name: Value, arg_count: u8) -> bool {
        let receiver = self.peek(arg_count as usize).unwrap().clone();

        match receiver {
            Value::Instance(instance) => {
                if let Some(value) = instance.read().fields.read().get(&name.to_string()) {
                    self.stack.pop();
                    return self.call_value(value.clone(), arg_count);
                }

                self.invoke_from_class(instance.read().clone().class, name, arg_count)
            }
            _ => {
                self.runtime_error("Only instances have methods");
                false
            }
        }
    }

    fn invoke_from_class(
        &mut self,
        class: Rc<RwLock<value::Class>>,
        name: Value,
        arg_count: u8,
    ) -> bool {
        if let Some(method) = class.read().methods.read().get(&name.to_string()) {
            self.call(method.clone(), arg_count, false);
            true
        } else {
            self.runtime_error(format!("Undefined property '{}'", name).as_str());
            false
        }
    }

    fn bind_method(&mut self, value: Rc<RwLock<value::Value>>, name: Value) -> bool {
        match &*value.read() {
            Value::Instance(instance) => {
                if let Some(method) = instance
                    .read()
                    .class
                    .read()
                    .methods
                    .read()
                    .get(&name.to_string())
                {
                    let bound_method = Value::BoundMethod(Rc::new(RwLock::new(
                        value::BoundMethod::new(value.clone(), method.clone()),
                    )));
                    self.pop();
                    self.push(bound_method);
                    true
                } else {
                    self.runtime_error(format!("Undefined property '{}'", name).as_str());
                    false
                }
            }
            _ => {
                self.runtime_error("Only instances have methods");
                false
            }
        }
    }

    fn define_method(&mut self, name: Value) {
        let method = self.peek(0).unwrap().clone();
        let class = self.peek(1).unwrap().clone();
        match (class, method) {
            (Value::Class(class), Value::Closure(method)) => {
                class
                    .write()
                    .methods
                    .write()
                    .insert(name.to_string(), method);
            }
            _ => {
                self.runtime_error("Only classes have methods");
                return;
            }
        }
        self.pop();
    }

    fn close_up_values(&mut self) {
        let frame = self.frames.last().unwrap();
        let mut i = 0;
        for up_value in frame.closure.read().up_values.read().iter() {
            let mut up_value = up_value.write();
            if up_value.location == Value::Nil {
                up_value.location = frame.slots[i].clone();
                up_value.closed = true;
            }
            i += 1;
        }
    }

    fn capture_up_value(&mut self, local: Value) -> Rc<RwLock<value::UpValueObject>> {
        let last_frame = self.frames.last_mut().unwrap();
        for up_value in last_frame.closure.read().up_values.read().iter() {
            if up_value.read().location == local {
                return up_value.clone();
            }
        }

        let up_value = Rc::new(RwLock::new(value::UpValueObject::new(Value::Nil)));
        last_frame
            .closure
            .read()
            .up_values
            .write()
            .push(up_value.clone());
        up_value.write().location = local;
        up_value.write().closed = false;
        up_value
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> bool {
        match callee {
            Value::BoundMethod(bound_method) => {
                let bound_method = bound_method.write();
                let method = bound_method.method.clone();
                let receiver = bound_method.receiver.clone();

                let frame = self.frames.last_mut().unwrap();

                frame.slots.insert(
                    frame.slots.len() - arg_count as usize,
                    receiver.read().clone(),
                );

                self.call(method, arg_count, false)
            }
            Value::Closure(closure) => {
                let frame = self.frames.last_mut().unwrap();

                frame
                    .slots
                    .insert(frame.slots.len() - arg_count as usize, Value::Nil);

                self.call(closure, arg_count, false)
            }
            Value::Class(class) => {
                self.stack.pop();
                let instance = Rc::new(RwLock::new(value::Instance::new(class.clone())));

                let class = class.read();
                let methods = class.methods.read();
                let initializer = methods.get("init");

                match initializer {
                    Some(initializer) => {
                        let frame = self.frames.last_mut().unwrap();

                        frame.slots.insert(
                            frame.slots.len() - arg_count as usize,
                            Value::Instance(instance.clone()),
                        );

                        if !self.call(initializer.clone(), arg_count, true) {
                            return false;
                        }
                    }
                    None => {
                        if arg_count != 0 {
                            self.runtime_error("Expected 0 arguments but got 1");
                            return false;
                        }
                    }
                }

                self.push(Value::Instance(instance.clone()));
                self.pop();

                true
            }
            Value::NativeFunction(function) => {
                let result = self.native_call(function, arg_count);

                let frame = self.frames.last_mut().unwrap();
                frame.slots.truncate(frame.slots.len() - arg_count as usize);

                self.pop();
                self.push(result);
                true
            }
            _ => {
                self.runtime_error("Can only call functions and classes");
                false
            }
        }
    }

    fn native_call(&mut self, function: Rc<RwLock<value::NativeFunction>>, arg_count: u8) -> Value {
        let mut args = Vec::new();
        for _ in 0..arg_count {
            args.push(self.pop().unwrap());
        }
        args.reverse();
        (function.read().function)(args)
    }

    fn call(&mut self, closure: Rc<RwLock<value::Closure>>, arg_count: u8, is_init: bool) -> bool {
        if arg_count != closure.read().function.read().arity as u8 {
            self.runtime_error(
                format!(
                    "Expected {} arguments but got {}",
                    closure.read().function.read().arity,
                    arg_count
                )
                .as_str(),
            );
            return false;
        }

        let frame = self.frames.last_mut().unwrap();

        if !is_init {
            frame
                .slots
                .insert(frame.slots.len() - arg_count as usize, Value::Nil);
        }

        let slots = frame
            .slots
            .split_off(frame.slots.len() - arg_count as usize - 1);

        self.frames.push(CallFrame {
            closure,
            ip: 0,
            slots,
        });

        true
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);

        for frame in self.frames.iter().rev() {
            let function = frame.closure.read().function.clone();
            let function = function.read();
            let chunk = function.chunk.read();
            let instruction = chunk.code[frame.ip - 1];
            let line = chunk.lines[frame.ip - 1];
            eprintln!("[line {}] in {}", line, function.name);

            match OpCode::from(instruction) {
                OpCode::Call => eprintln!("    called here"),
                OpCode::Closure => eprintln!("    defined here"),
                _ => (),
            }
        }

        let mut frame = self.frames.last_mut().unwrap();

        self.stack.truncate(frame.slots.len());

        if self.frames.len() == 1 {
            self.stack.pop();
        } else {
            self.frames.pop();
            frame = self.frames.last_mut().unwrap();
            frame.ip += 1;
        }
    }

    fn define_native(
        &mut self,
        name: String,
        function: Box<fn(Vec<Value>) -> Value>,
        arity: usize,
    ) {
        self.stack.push(Value::String(name.clone()));
        let native_function = Rc::new(RwLock::new(value::NativeFunction::new(
            name.clone(),
            arity,
            function,
        )));
        self.stack
            .push(Value::NativeFunction(native_function.clone()));
        self.globals
            .insert(name.clone(), Value::NativeFunction(native_function));
        self.stack.pop();
        self.stack.pop();
    }

    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut();
        match frame {
            Some(frame) => {
                let function = frame.closure.read().function.clone();
                let function = function.read();
                let byte = function.chunk.read().code[frame.ip];
                frame.ip += 1;
                byte
            }
            None => panic!("Expected frame"),
        }
    }

    #[inline(always)]
    fn read_constant(&mut self) -> Value {
        let frame = self.frames.last_mut();
        match frame {
            Some(frame) => {
                let constant = frame.closure.read().function.read().chunk.read().code[frame.ip];
                frame.ip += 1;
                frame.closure.read().function.read().chunk.read().constants[constant as usize]
                    .clone()
            }
            None => panic!("Expected frame"),
        }
    }

    #[inline(always)]
    fn read_short(&mut self) -> u16 {
        let frame = self.frames.last_mut();
        match frame {
            Some(frame) => {
                let function = frame.closure.read().function.clone();
                let function = function.read();
                let byte1 = function.chunk.read().code[frame.ip];
                let byte2 = function.chunk.read().code[frame.ip + 1];
                frame.ip += 2;
                (byte1 as u16) << 8 | byte2 as u16
            }
            None => panic!("Expected frame"),
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
