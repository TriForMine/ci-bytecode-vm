use std::sync::Arc;
use crate::chunk::Chunk;
use parking_lot::RwLock;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
    String(String),
    Function(Arc<RwLock<Function>>),
    NativeFunction(Arc<RwLock<NativeFunction>>),
    RunTimeError(String),
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
pub struct Function {
    pub arity: usize,
    pub chunk: Arc<RwLock<Chunk>>,
    pub name: String,
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
            chunk: Arc::new(RwLock::new(Chunk::new())),
            name,
        }
    }

    pub fn new_script() -> Self {
        Function {
            arity: 0,
            chunk: Arc::new(RwLock::new(Chunk::new())),
            name: String::from("script"),
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
            Value::NativeFunction(func) => write!(f, "<native fn {}>", func.read().name),
            Value::RunTimeError(s) => write!(f, "{}", s),
        }
    }
}