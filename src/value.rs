use std::sync::Arc;
use crate::chunk::Chunk;
use parking_lot::RwLock;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    String(String),
    Function(Arc<RwLock<Function>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
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

#[derive(PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

impl FunctionType {
    pub fn replace(&mut self, function_type: FunctionType) {
        match self {
            FunctionType::Function => {
                *self = function_type;
            }
            FunctionType::Script => {
                *self = function_type;
            }
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Bool(b) => !b,
            Value::Number(n) => *n == 0.0,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Value::Nil => true,
            _ => false,
        }
    }

    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            _ => panic!("Value is not a number"),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(func) => write!(f, "<fn {}>", func.read().name),
        }
    }
}
