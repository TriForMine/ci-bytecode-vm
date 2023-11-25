use std::rc::Rc;
use crate::chunk::Chunk;
use parking_lot::RwLock;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
    String(String),
    Function(Rc<RwLock<Function>>),
    Closure(Rc<RwLock<Closure>>),
    NativeFunction(Rc<RwLock<NativeFunction>>),
    RunTimeError(String),
}

#[derive(Clone, Debug)]
pub struct UpValueObject {
    pub location: Value,
    pub closed: bool,
}

impl PartialEq for UpValueObject {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl UpValueObject {
    pub fn new(location: Value) -> Self {
        UpValueObject { location, closed: false }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            (Value::Int(i1), Value::Int(i2)) => i1 == i2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Nil, Value::Nil) => true,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Function(f1), Value::Function(f2)) => {
                let f1 = f1.read();
                let f2 = f2.read();
                f1.eq(&f2)
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub function: Rc<RwLock<Function>>,
    pub up_values: Rc<RwLock<Vec<Rc<RwLock<UpValueObject>>>>>,
}

#[derive(Clone, Debug, Copy)]
pub struct Upvalue {
    pub index: u8,
    pub is_local: bool,
}

impl Closure {
    pub fn new(function: Rc<RwLock<Function>>) -> Self {
        Closure {
            function,
            up_values: Rc::new(RwLock::new(Vec::new())),
        }
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.function.read().name == other.function.read().name
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub arity: usize,
    pub chunk: Rc<RwLock<Chunk>>,
    pub name: String,
    pub up_value_count: u8,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Function {
    pub fn new(name: String) -> Self {
        Function {
            arity: 0,
            chunk: Rc::new(RwLock::new(Chunk::new())),
            name,
            up_value_count: 0,
        }
    }

    pub fn new_script() -> Self {
        Function {
            arity: 0,
            chunk: Rc::new(RwLock::new(Chunk::new())),
            name: String::from("script"),
            up_value_count: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub function: Box<fn(Vec<Value>) -> Value>,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl NativeFunction {
    pub fn new(name: String, arity: usize, function: Box<fn(Vec<Value>) -> Value>) -> Self {
        NativeFunction {
            name,
            arity,
            function,
        }
    }
}

#[derive(PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

impl Value {
    pub fn is_falsely(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Bool(b) => !b,
            Value::Int(i) => *i == 0,
            Value::Float(f) => *f == 0.0,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{:.?}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(func) => write!(f, "<fn {}>", func.read().name),
            Value::Closure(closure) => write!(f, "<fn {}>", closure.read().function.read().name),
            Value::NativeFunction(func) => write!(f, "<native fn {}>", func.read().name),
            Value::RunTimeError(s) => write!(f, "{}", s),
        }
    }
}